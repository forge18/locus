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
    sync::{Arc, Mutex, OnceLock, RwLock},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::{
    harness::{
        materialize::extensions::{ExtensionEntry, ExtensionSet},
        registry::{load_from_directory, HarnessDefinition, HarnessRegistry},
    },
    ids::{ProjectId, RoutineId, RunId, TaskId},
    ipc::{EventChannel, PtyChannel},
    lsp::{LanguageCatalog, LspHost},
    plugin::{builtin_manifests, PluginKind, PluginProcess, WorkItemProviderDescriptor},
    repo::RepoManager,
    runtime::{
        backend::{RuntimeBackend, RuntimeConfig},
        container::{ContainerRuntime, DockerContainerRuntime},
        daemon::Daemon,
        dap::DebugSessionRegistry,
        run::{self, SpawnRequest},
        session::{Artifact, PermissionPosture, Run, RunStatus},
    },
    sandbox::{
        credential_proxy::CredentialProxy,
        egress::{DestinationAllowlists, EgressTier},
        image::ToolPin,
        sbx::SbxContainerRuntime,
    },
    services::{
        agents::AgentDefinition, bots::RoutineClaimResult, handoff::HandoffRegistry,
        telemetry::EventCollector, tools::RoleToolScope,
    },
    store::Store,
    work_item::{
        pull_from_plugin, sync_capability_from_plugin, CompletionDelivery, CompletionEvent,
        CompletionOutbox, WorkItemRegistry, WorkItemSnapshot, WorkItemSyncApplication,
    },
};

/// How much fan-out each in-process channel buffers before a slow subscriber lags.
const CHANNEL_CAPACITY: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreHealthStatus {
    NotConfigured,
    Connecting,
    Connected,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreHealth {
    pub status: StoreHealthStatus,
    pub message: Option<String>,
}

impl StoreHealth {
    fn initial() -> Self {
        let configured = std::env::var("DATABASE_URL")
            .ok()
            .is_some_and(|url| !url.trim().is_empty());
        Self {
            status: if configured {
                StoreHealthStatus::Connecting
            } else {
                StoreHealthStatus::NotConfigured
            },
            message: None,
        }
    }
}

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
    store_health: RwLock<StoreHealth>,
    daemon: Arc<tokio::sync::Mutex<Daemon>>,
    credential_proxy: CredentialProxy,
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
            store_health: RwLock::new(StoreHealth::initial()),
            daemon: Arc::new(tokio::sync::Mutex::new(Daemon::with_debug(debug))),
            credential_proxy: CredentialProxy::new(
                std::env::var("LOCUS_CREDENTIAL_SECRET")
                    .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
                    .unwrap_or_default(),
                "model-provider",
            ),
        }))
    }

    /// Attach the store. Separate from [`Core::load`] because the desktop shell starts
    /// before Postgres is reachable, and the registry surfaces do not need it.
    pub async fn connect(&self, database_url: &str) -> Result<&Store> {
        if let Some(store) = self.store.get() {
            self.set_store_health(StoreHealth {
                status: StoreHealthStatus::Connected,
                message: None,
            });
            return Ok(store);
        }
        let _connect_lock = self.connect_lock.lock().await;
        if let Some(store) = self.store.get() {
            self.set_store_health(StoreHealth {
                status: StoreHealthStatus::Connected,
                message: None,
            });
            return Ok(store);
        }
        self.set_store_health(StoreHealth {
            status: StoreHealthStatus::Connecting,
            message: None,
        });
        let result: Result<&Store> = async {
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
        .await;
        match &result {
            Ok(_) => self.set_store_health(StoreHealth {
                status: StoreHealthStatus::Connected,
                message: None,
            }),
            Err(error) => {
                tracing::warn!(%error, "Locus store connection failed");
                self.set_store_health(StoreHealth {
                    status: StoreHealthStatus::Unavailable,
                    message: Some("Locus store is unavailable".into()),
                });
            }
        }
        result
    }

    fn set_store_health(&self, health: StoreHealth) {
        if let Ok(mut current) = self.store_health.write() {
            *current = health;
        }
    }

    pub fn store_health(&self) -> StoreHealth {
        self.store_health
            .read()
            .map(|health| health.clone())
            .unwrap_or(StoreHealth {
                status: StoreHealthStatus::Unavailable,
                message: Some("store health lock is poisoned".into()),
            })
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

    pub fn daemon(&self) -> &tokio::sync::Mutex<Daemon> {
        self.daemon.as_ref()
    }

    /// Deliver a prompt to an already-running ACP conversation.
    pub async fn prompt_run(&self, run_id: RunId, prompt: impl Into<String>) -> Result<()> {
        self.daemon.lock().await.prompt_run(run_id, prompt)
    }

    /// Launch one queue-claimed run from durable state. All inputs that affect
    /// the container are resolved by the host from the run, its session, and
    /// the project settings; no queue caller can substitute them.
    pub async fn fire_due_bot_routines(
        &self,
        seen_minutes: &mut BTreeMap<RoutineId, i64>,
        runtime: &mut dyn ContainerRuntime,
    ) -> Result<usize> {
        let Some(store) = self.store() else {
            return Ok(0);
        };
        let now = time::OffsetDateTime::now_utc();
        let minute = now.unix_timestamp() / 60;
        let default_model =
            std::env::var("LOCUS_DEFAULT_MODEL_ID").unwrap_or_else(|_| "unconfigured-model".into());
        let mut started = 0;
        for routine in store.all_bot_routines().await? {
            if !routine.enabled || seen_minutes.get(&routine.id) == Some(&minute) {
                continue;
            }
            let cron = match crate::services::schedule::CronExpression::parse(
                &routine.cron_expression,
            ) {
                Ok(cron) => cron,
                Err(error) => {
                    tracing::warn!(routine = %routine.id, %error, "skip invalid bot routine cron");
                    seen_minutes.insert(routine.id, minute);
                    continue;
                }
            };
            if !cron.matches(now) {
                continue;
            }
            seen_minutes.insert(routine.id, minute);
            let claim = store
                .fire_bot_routine(routine.id, now, &default_model)
                .await?;
            if let RoutineClaimResult::Started(start) = claim {
                let run_id = store
                    .active_bot_run(start.bot_id)
                    .await?
                    .ok_or_else(|| anyhow!("bot routine started without an active run"))?;
                let dispatch = store
                    .dispatch_run(run_id)
                    .await?
                    .ok_or_else(|| anyhow!("bot routine run disappeared before launch"))?;
                if let Err(error) = self.spawn_dispatch_run(store, dispatch, runtime).await {
                    let _ = store
                        .finish_bot_run(start.bot_id, run_id, false, None)
                        .await;
                    let _ = store
                        .complete_bot_routine_execution(
                            start.execution_id,
                            crate::services::bots::RoutineResult::failed(error.to_string()),
                            Some(run_id),
                        )
                        .await;
                    return Err(error);
                }
                started += 1;
            }
        }
        Ok(started)
    }

    pub async fn dispatch_once(&self, runtime: &mut dyn ContainerRuntime) -> Result<Vec<RunId>> {
        let Some(store) = self.store() else {
            return Ok(Vec::new());
        };
        let claimed = store.claim_dispatchable_runs().await?;
        let mut started = Vec::new();
        for run_id in claimed {
            let Some(dispatch) = store.dispatch_run(run_id).await? else {
                store
                    .abort_dispatch_run(run_id, "dispatch run disappeared before launch")
                    .await?;
                continue;
            };
            match self.spawn_dispatch_run(store, dispatch, runtime).await {
                Ok(_) => started.push(run_id),
                Err(error) => {
                    tracing::warn!(%run_id, %error, "dispatch run failed to start");
                    store.abort_dispatch_run(run_id, &error.to_string()).await?;
                }
            }
        }
        Ok(started)
    }

    pub async fn spawn_dispatch_run(
        &self,
        store: &Store,
        dispatch: crate::store::dispatch::DispatchRun,
        runtime: &mut dyn ContainerRuntime,
    ) -> Result<run::SpawnedRun> {
        let project_settings = store.project_settings(dispatch.project_id.into()).await?;
        let project_id = dispatch.project_id.to_string();
        let harness = select_dispatch_harness(
            &self.registry,
            &project_settings,
            dispatch.harness.as_deref(),
        )?;
        let frontmatter = serde_json::from_value(dispatch.agent_frontmatter.clone())?;
        let definition = AgentDefinition {
            frontmatter,
            body: dispatch.agent_body,
            warnings: Vec::new(),
        };
        let mut extensions = ExtensionSet::default();
        extensions.insert("agents", vec![definition.extension_entry()?]);
        if let Some(base_context) = project_settings.base_context() {
            extensions.insert(
                "context",
                vec![ExtensionEntry::new(
                    "project-base-context.md",
                    serde_json::json!({"name": "project-base-context"}),
                    base_context,
                )],
            );
        }
        let tools = definition
            .frontmatter
            .tools
            .iter()
            .map(|name| ToolPin {
                name: name.clone(),
                version: "catalog".into(),
            })
            .collect();
        let guardrails = store.guardrail_defaults().await?;
        let egress_tier = match guardrails.network_tier {
            crate::runtime::dispatch::NetworkTier::Closed => EgressTier::None,
            crate::runtime::dispatch::NetworkTier::Internal => EgressTier::Model,
            crate::runtime::dispatch::NetworkTier::Open => EgressTier::Open,
        };
        let run_id: RunId = dispatch.run_id.into();
        let mut run = Run {
            id: run_id,
            session_id: dispatch.session_id.into(),
            resolved_model_id: dispatch.resolved_model_id.clone(),
            status: match dispatch.status.as_str() {
                "queued" => RunStatus::Queued,
                "running" => RunStatus::Running,
                status => bail!("dispatch run `{run_id}` has non-launchable status `{status}`"),
            },
            permission_posture: PermissionPosture::parse(&dispatch.permission_posture)
                .context("parse persisted run permission posture")?,
            events: Vec::new(),
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };
        let workspace_remote = dispatch
            .workspace_remote
            .ok_or_else(|| anyhow!("dispatch run has no project local remote"))?;
        let config_root = std::env::var_os("LOCUS_RUN_CONFIG_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("locus/config"))
            .join(run_id.to_string());
        let policy_root = std::env::var_os("LOCUS_EGRESS_POLICY_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("locus/egress-policies"));
        let socket_source = std::env::var_os("LOCUS_SOCKET_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/run/locus.sock"));
        if dispatch.branch.starts_with("interact/") {
            RepoManager::default().ensure_interact_branch(&workspace_remote, &dispatch.branch)?;
        } else if let Some(bot_id) = dispatch.branch.strip_prefix("bots/") {
            RepoManager::default().ensure_bot_branch(&workspace_remote, bot_id)?;
        }
        let request = SpawnRequest {
            project_id: &project_id,
            harness,
            extensions: &extensions,
            config_root,
            socket_source,
            workspace_remote,
            workspace_branch: (dispatch.branch.starts_with("bots/")
                || dispatch.branch.starts_with("interact/"))
            .then_some(dispatch.branch.as_str()),
            credential_proxy: run::CredentialProxyConfig::new(
                "http://host.docker.internal:44000/",
            )?,
            credential_proxy_authorizer: &self.credential_proxy,
            audit_store: store.clone(),
            egress_tier,
            egress_allowlists: DestinationAllowlists::new(
                ["api.anthropic.com"],
                std::iter::empty::<&str>(),
            ),
            egress_policy_root: policy_root,
            run_nonce: uuid::Uuid::new_v4().to_string(),
            lsp_enabled: definition
                .frontmatter
                .tools
                .iter()
                .any(|tool| tool == "lsp"),
            base_image_digest: format!("{}:{}", harness.image.base, harness.image.version),
            tools,
            project_extension_scope: project_settings.extension_overrides().clone(),
            project_tool_scope: project_settings.tool_scope().clone(),
            role_tool_scope: RoleToolScope::default(),
            project_settings,
            handoff_context: None,
            plugin: None,
            collector: &self.collector,
        };
        let spawned = self
            .daemon
            .lock()
            .await
            .spawn_run(store, &mut run, request, runtime)
            .await?;
        if let Some(session) = spawned.acp_session.clone() {
            let store = store.clone();
            let daemon = self.daemon.clone();
            let bot_context = store.bot_run_context(run_id).await?;
            tokio::spawn(async move {
                session.wait_closed().await;
                if let Some(context) = bot_context {
                    if let Err(error) = store
                        .finish_bot_run(context.bot_id, run_id, true, None)
                        .await
                    {
                        tracing::warn!(%run_id, %error, "persist completed bot run");
                    }
                    if let Some(execution_id) = context.routine_execution_id {
                        let result =
                            crate::services::bots::RoutineResult::passed("ACP session completed");
                        if let Err(error) = store
                            .complete_bot_routine_execution(execution_id, result, Some(run_id))
                            .await
                        {
                            tracing::warn!(%run_id, %error, "persisted bot routine completion failed");
                        }
                    }
                } else if let Err(error) = store.complete_dispatch_run(run_id, 0).await {
                    tracing::warn!(%run_id, %error, "persist completed dispatch run");
                }
                daemon.lock().await.finish_run(run_id);
            });
        }
        Ok(spawned)
    }
}

fn select_dispatch_harness<'a>(
    registry: &'a HarnessRegistry,
    settings: &crate::services::project::ProjectSettings,
    requested: Option<&str>,
) -> Result<&'a HarnessDefinition> {
    let requested = requested
        .filter(|name| !name.eq_ignore_ascii_case("any"))
        .or_else(|| settings.agent_default())
        .or_else(|| settings.harness_allow_list().first().map(String::as_str));
    let harness = requested
        .and_then(|name| registry.by_name(name))
        .or_else(|| registry.iter().next())
        .ok_or_else(|| anyhow!("no harness is registered"))?;
    if !settings.harness_allow_list().is_empty() && !settings.permits_harness(&harness.name) {
        bail!("harness `{}` is not allowed for the project", harness.name)
    }
    Ok(harness)
}
