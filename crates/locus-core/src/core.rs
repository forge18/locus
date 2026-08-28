//! The composition root: the one place the subsystems in PLAN.md §Process topology are
//! assembled.
//!
//! Before this, nothing built the graph. The desktop host hand-constructed three leaf
//! objects as Tauri state, never held a `Store`, and re-read and re-parsed the whole
//! harness registry from disk on every invoke. Each new consumer wired again, so the
//! desktop host and the socket router were free to drift.
//!
//! `Core` is built once and shared as `Arc<Core>`: Tauri manages it, `locusd` serves the
//! agent socket from it, and both see the same registry, the same collector, and the same
//! store.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    harness::registry::{load_from_directory, HarnessRegistry},
    ids::{ProjectId, TaskId},
    ipc::{EventChannel, PtyChannel},
    lsp::{LanguageCatalog, LspHost},
    plugin::{builtin_manifests, PluginKind, PluginProcess, WorkItemProviderDescriptor},
    runtime::{
        backend::{RuntimeBackend, RuntimeConfig},
        container::{ContainerRuntime, DockerContainerRuntime},
        daemon::Daemon,
        dap::DebugSessionRegistry,
    },
    sandbox::sbx::SbxContainerRuntime,
    services::{handoff::HandoffRegistry, telemetry::EventCollector},
    store::Store,
    work_item::{
        pull_from_plugin, sync_capability_from_plugin, CompletionDelivery, CompletionEvent,
        CompletionOutbox, WorkItemRegistry, WorkItemSnapshot, WorkItemSyncApplication,
    },
};

/// How much fan-out each in-process channel buffers before a slow subscriber lags.
const CHANNEL_CAPACITY: usize = 1_024;

/// Everything that outlives a window.
///
/// PLAN.md §Process topology: "`locusd` outlives the window. It runs as a background
/// service; closing the app detaches the UI and nothing else."
pub struct Core {
    registry: HarnessRegistry,
    runtime: RuntimeConfig,
    collector: EventCollector,
    pty: PtyChannel,
    events: EventChannel,
    lsp: LspHost,
    debug: DebugSessionRegistry,
    handoffs: Arc<Mutex<HandoffRegistry>>,
    work_items: Mutex<WorkItemRegistry>,
    completion_outbox: tokio::sync::Mutex<CompletionOutbox>,
    /// Serializes first-time database connection and hydration. `Core` is shared as `Arc`, so
    /// the store cannot be assigned through `&mut self`.
    connect_lock: tokio::sync::Mutex<()>,
    work_item_operation_lock: tokio::sync::Mutex<()>,
    pending_work_item_previews: Mutex<BTreeMap<TaskId, (ProjectId, WorkItemSnapshot)>>,
    /// Set once, by [`Core::connect`].
    store: OnceLock<Store>,
    daemon: Mutex<Daemon>,
}

impl Core {
    /// Assemble everything that does not need a database.
    ///
    /// The registry is loaded once here rather than per request: it is the harness
    /// contract, and re-parsing eleven TOMLs on every invoke is work the process already
    /// did at start.
    pub fn load(harnesses: impl AsRef<Path>) -> Result<Arc<Self>> {
        let registry =
            load_from_directory(harnesses.as_ref()).context("load the harness registry")?;
        let mut language_catalog = LanguageCatalog::builtin().context("load language catalog")?;
        if let Some(root) = std::env::var_os("LOCUS_LSP_USER_CATALOG") {
            let root = Path::new(&root);
            if root.exists() {
                language_catalog
                    .merge_user_catalog(LanguageCatalog::load_user_catalog(root)?)
                    .context("merge user language catalog")?;
            }
        }
        let runtime = RuntimeConfig::from_env().context("load container runtime config")?;
        let debug = DebugSessionRegistry::default();
        Ok(Arc::new(Self {
            registry,
            runtime,
            collector: EventCollector::new(CHANNEL_CAPACITY),
            pty: PtyChannel::new(CHANNEL_CAPACITY),
            events: EventChannel::new(CHANNEL_CAPACITY),
            lsp: LspHost::new(language_catalog),
            debug: debug.clone(),
            handoffs: Arc::new(Mutex::new(HandoffRegistry::default())),
            work_items: Mutex::new(WorkItemRegistry::default()),
            completion_outbox: tokio::sync::Mutex::new(CompletionOutbox::default()),
            connect_lock: tokio::sync::Mutex::new(()),
            work_item_operation_lock: tokio::sync::Mutex::new(()),
            pending_work_item_previews: Mutex::new(BTreeMap::new()),
            store: OnceLock::new(),
            daemon: Mutex::new(Daemon::with_debug(debug)),
        }))
    }

    /// Attach the store. Separate from [`Core::load`] because the desktop shell starts
    /// before Postgres is reachable, and the registry surfaces do not need it.
    pub async fn connect(&self, database_url: &str) -> Result<&Store> {
        if let Some(store) = self.store.get() {
            return Ok(store);
        }
        let _connect_lock = self.connect_lock.lock().await;
        if let Some(store) = self.store.get() {
            return Ok(store);
        }
        let store = Store::connect(database_url)
            .await
            .context("connect the Locus store")?;
        let configs = store
            .load_external_work_item_providers()
            .await
            .context("hydrate external work-item providers")?;
        let imported = store
            .load_external_work_items()
            .await
            .context("hydrate imported work items")?;
        let completions = store
            .load_external_completions()
            .await
            .context("hydrate external completion outbox")?;

        let mut work_item_registry = WorkItemRegistry::default();
        for config in configs {
            work_item_registry.configure(config);
        }
        for item in imported {
            work_item_registry
                .restore_imported_with_sync_state(
                    item.task,
                    item.snapshot,
                    item.workflow,
                    item.runs,
                    item.evidence,
                    item.sync_state,
                )
                .map_err(|error| anyhow!("restore imported work item: {error}"))?;
        }
        let mut completion_outbox = CompletionOutbox::default();
        for item in completions {
            completion_outbox
                .restore_delivery(CompletionDelivery {
                    event: CompletionEvent {
                        id: item.id,
                        task_id: item.task_id,
                        locator: item.locator,
                        evidence: item.evidence,
                        comment: item.comment,
                    },
                    attempts: item.attempts,
                    commented: item.commented,
                    resolved: item.resolved,
                })
                .map_err(|error| anyhow!("restore external completion: {error}"))?;
        }

        *self
            .work_items
            .lock()
            .map_err(|_| anyhow!("external work-item registry lock is poisoned"))? =
            work_item_registry;
        *self.completion_outbox.lock().await = completion_outbox;
        self.store
            .set(store)
            .map_err(|_| anyhow!("Locus store was connected concurrently"))?;
        self.store
            .get()
            .ok_or_else(|| anyhow!("Locus store was not initialized"))
    }

    pub fn registry(&self) -> &HarnessRegistry {
        &self.registry
    }

    pub fn runtime_config(&self) -> &RuntimeConfig {
        &self.runtime
    }

    /// Connect the selected host runtime. Docker callers may choose to degrade when their
    /// daemon is unavailable; sbx callers must propagate the error and never fall back.
    pub fn connect_container_runtime(&self) -> Result<Box<dyn ContainerRuntime>> {
        match self.runtime.backend {
            RuntimeBackend::Docker => Ok(Box::new(DockerContainerRuntime::connect()?)),
            RuntimeBackend::Sbx => Ok(Box::new(SbxContainerRuntime::connect(
                self.runtime.sbx.clone(),
            )?)),
        }
    }

    pub fn collector(&self) -> &EventCollector {
        &self.collector
    }

    pub fn pty(&self) -> &PtyChannel {
        &self.pty
    }

    pub fn events(&self) -> &EventChannel {
        &self.events
    }

    pub fn lsp(&self) -> &LspHost {
        &self.lsp
    }

    pub fn debug(&self) -> &DebugSessionRegistry {
        &self.debug
    }

    pub fn handoffs(&self) -> Arc<Mutex<HandoffRegistry>> {
        self.handoffs.clone()
    }

    pub fn work_items(&self) -> &Mutex<WorkItemRegistry> {
        &self.work_items
    }

    pub fn completion_outbox(&self) -> &tokio::sync::Mutex<CompletionOutbox> {
        &self.completion_outbox
    }

    pub fn work_item_operation_lock(&self) -> &tokio::sync::Mutex<()> {
        &self.work_item_operation_lock
    }

    pub fn pending_work_item_previews(
        &self,
    ) -> &Mutex<BTreeMap<TaskId, (ProjectId, WorkItemSnapshot)>> {
        &self.pending_work_item_previews
    }

    /// The store, once [`Core::connect`] has run. `None` means the shell is up but
    /// Postgres is not, which is a state the UI is expected to render rather than crash on.
    pub fn store(&self) -> Option<&Store> {
        self.store.get()
    }

    /// Pull and fold one sync-capable external work item. The daemon and desktop host use
    /// the same path so a window is not required for scheduled synchronization.
    pub async fn sync_external_work_item(
        &self,
        task_id: TaskId,
    ) -> Result<WorkItemSyncApplication> {
        let store = self
            .store()
            .ok_or_else(|| anyhow!("Locus store is not connected"))?;
        let (identity, cursor, before_registry) = {
            let registry = self
                .work_items
                .lock()
                .map_err(|_| anyhow!("external work-item registry lock is poisoned"))?;
            let task = registry
                .board()
                .task(task_id)
                .ok_or_else(|| anyhow!("task `{task_id}` was not found"))?;
            let identity = task
                .external_work_item
                .as_ref()
                .ok_or_else(|| anyhow!("task is not an imported work item"))?
                .identity
                .clone();
            let cursor = registry
                .sync_state(&identity)
                .and_then(|state| state.pull_cursor.clone());
            (identity, cursor, registry.clone())
        };
        let manifest = builtin_manifests()
            .into_iter()
            .find(|manifest| {
                manifest.kind == PluginKind::Provider && manifest.id == identity.plugin_id.as_str()
            })
            .ok_or_else(|| anyhow!("work-item plugin is not admitted"))?;
        let catalog = WorkItemProviderDescriptor::from_manifest(&manifest)?;
        if !catalog.sync {
            bail!("external work-item provider does not support synchronization")
        }
        let executable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&manifest.executable);
        let process = PluginProcess::spawn(executable, Duration::from_secs(30)).await?;
        let result: Result<WorkItemSyncApplication> = async {
            let handshake = process
                .handshake(
                    &[
                        crate::work_item::WORK_ITEM_SNAPSHOT_CAPABILITY.into(),
                        crate::work_item::WORK_ITEM_COMMENT_CAPABILITY.into(),
                    ],
                    &[
                        crate::work_item::WORK_ITEM_SNAPSHOT_CAPABILITY,
                        crate::work_item::WORK_ITEM_COMMENT_CAPABILITY,
                        crate::work_item::WORK_ITEM_RESOLVE_CAPABILITY,
                        crate::work_item::WORK_ITEM_SYNC_CAPABILITY,
                    ],
                )
                .await?;
            let runtime =
                WorkItemProviderDescriptor::from_plugin_descriptor(&handshake.descriptor)?;
            let catalog_descriptor = catalog.plugin_descriptor();
            if runtime.manifest.protocol != catalog.manifest.protocol
                || runtime.manifest.kind != catalog.manifest.kind
                || runtime.manifest.id != catalog.manifest.id
                || runtime.manifest.version != catalog.manifest.version
                || runtime.manifest.capabilities != catalog.manifest.capabilities
                || runtime.manifest.permissions != catalog.manifest.permissions
                || handshake.descriptor.schema_versions != catalog_descriptor.schema_versions
            {
                bail!("work-item plugin runtime does not match its admitted manifest")
            }
            let provider = runtime.work_item_provider()?;
            let capability = sync_capability_from_plugin(&process).await?;
            let pull = pull_from_plugin(&process, &identity, cursor).await?;
            let synced_at = crate::services::telemetry::now_timestamp();
            let (application, snapshot, state) = {
                let mut registry = self
                    .work_items
                    .lock()
                    .map_err(|_| anyhow!("external work-item registry lock is poisoned"))?;
                let mut application =
                    registry.apply_pull(&identity, &capability, pull, &synced_at)?;
                application.resolution_supported = provider.capabilities.resolve;
                let imported = registry
                    .imported(&identity)
                    .ok_or_else(|| anyhow!("imported work item disappeared during sync"))?;
                (
                    application,
                    imported.snapshot.clone(),
                    imported.sync_state.clone(),
                )
            };
            store
                .persist_external_sync(task_id, &snapshot, &application, &state)
                .await?;
            Ok(application)
        }
        .await;
        let _ = process.shutdown().await;
        if let Err(error) = &result {
            *self
                .work_items
                .lock()
                .map_err(|_| anyhow!("external work-item registry lock is poisoned"))? =
                before_registry;
            let _ = store
                .record_external_sync_error(task_id, &error.to_string())
                .await;
        }
        result
    }

    pub fn daemon(&self) -> &Mutex<Daemon> {
        &self.daemon
    }
}
