//! Provider-neutral external work-item import and one-way completion delivery.
//!
//! Import snapshots become local task state. Source edits never synchronize back
//! into that state; the only outbound operation is the idempotent completion event
//! emitted after local Done.

use std::collections::BTreeMap;

use crate::{
    ids::{ArtifactId, ProjectId, TaskId},
    services::{
        board::{BoardEvent, BoardProjection, BoardTask},
        manage::TaskColumn,
        task::{TaskOrchestrator, WorkflowSelection},
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct WorkItemProviderId(String);

impl WorkItemProviderId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkItemError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(WorkItemError::InvalidConfiguration);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemCapabilities {
    pub comments: bool,
    pub resolve: bool,
}

pub const WORK_ITEM_SNAPSHOT_METHOD: &str = "work_item.snapshot";
pub const WORK_ITEM_COMMENT_METHOD: &str = "work_item.comment";
pub const WORK_ITEM_RESOLVE_METHOD: &str = "work_item.resolve";
pub const WORK_ITEM_SNAPSHOT_CAPABILITY: &str = "work_item.snapshot";
pub const WORK_ITEM_COMMENT_CAPABILITY: &str = "work_item.comment";
pub const WORK_ITEM_RESOLVE_CAPABILITY: &str = "work_item.resolve";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemProviderConfig {
    pub plugin_id: WorkItemProviderId,
    pub host: String,
    pub project: String,
}

impl WorkItemProviderConfig {
    pub fn new(
        plugin_id: impl Into<String>,
        host: impl Into<String>,
        project: impl Into<String>,
    ) -> Result<Self, WorkItemError> {
        let config = Self {
            plugin_id: WorkItemProviderId::new(plugin_id)?,
            host: host.into(),
            project: project.into(),
        };
        if config.host.trim().is_empty() || config.project.trim().is_empty() {
            return Err(WorkItemError::InvalidConfiguration);
        }
        Ok(config)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct WorkItemIdentity {
    pub plugin_id: WorkItemProviderId,
    pub host: String,
    pub project: String,
    pub native_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemSnapshot {
    pub identity: WorkItemIdentity,
    pub url: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub status: String,
}

pub trait ExternalWorkItemProvider {
    fn plugin_id(&self) -> &WorkItemProviderId;
    fn capabilities(&self) -> WorkItemCapabilities;
    fn normalize(&self, snapshot: WorkItemSnapshot) -> Result<WorkItemSnapshot, WorkItemError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginWorkItemProvider {
    pub plugin_id: WorkItemProviderId,
    pub capabilities: WorkItemCapabilities,
}

impl PluginWorkItemProvider {
    pub fn new(
        plugin_id: impl Into<String>,
        capabilities: WorkItemCapabilities,
    ) -> Result<Self, WorkItemError> {
        Ok(Self {
            plugin_id: WorkItemProviderId::new(plugin_id)?,
            capabilities,
        })
    }
}

impl ExternalWorkItemProvider for PluginWorkItemProvider {
    fn plugin_id(&self) -> &WorkItemProviderId {
        &self.plugin_id
    }

    fn capabilities(&self) -> WorkItemCapabilities {
        self.capabilities
    }

    fn normalize(&self, snapshot: WorkItemSnapshot) -> Result<WorkItemSnapshot, WorkItemError> {
        snapshot.validate()?;
        if snapshot.identity.plugin_id != self.plugin_id {
            return Err(WorkItemError::ProviderIdentityMismatch);
        }
        Ok(snapshot)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemLookup {
    pub plugin_id: WorkItemProviderId,
    pub host: String,
    pub project: String,
    pub native_id: String,
}

impl From<&WorkItemIdentity> for WorkItemLookup {
    fn from(identity: &WorkItemIdentity) -> Self {
        Self {
            plugin_id: identity.plugin_id.clone(),
            host: identity.host.clone(),
            project: identity.project.clone(),
            native_id: identity.native_id.clone(),
        }
    }
}

pub async fn snapshot_from_plugin(
    process: &crate::plugin::PluginProcess,
    lookup: &WorkItemLookup,
) -> Result<WorkItemSnapshot, WorkItemError> {
    let response = process
        .call(
            WORK_ITEM_SNAPSHOT_METHOD,
            serde_json::to_value(lookup)
                .map_err(|error| WorkItemError::Plugin(error.to_string()))?,
        )
        .await
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    let snapshot: WorkItemSnapshot = serde_json::from_value(response)
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    if snapshot.identity.plugin_id != lookup.plugin_id
        || snapshot.identity.host != lookup.host
        || snapshot.identity.project != lookup.project
        || snapshot.identity.native_id != lookup.native_id
    {
        return Err(WorkItemError::ProviderIdentityMismatch);
    }
    snapshot.validate()?;
    Ok(snapshot)
}

impl WorkItemSnapshot {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        if self.identity.host.trim().is_empty()
            || self.identity.project.trim().is_empty()
            || self.identity.native_id.trim().is_empty()
            || self.url.trim().is_empty()
            || self.title.trim().is_empty()
        {
            return Err(WorkItemError::InvalidSnapshot);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalWorkState {
    Ready,
    InProgress,
    Testing,
    Reviewing,
    WaitingForApproval,
    Blocked,
    Paused,
    Cancelled,
    Failed,
    Done,
}

impl LocalWorkState {
    pub const ALL: [Self; 10] = [
        Self::Ready,
        Self::InProgress,
        Self::Testing,
        Self::Reviewing,
        Self::WaitingForApproval,
        Self::Blocked,
        Self::Paused,
        Self::Cancelled,
        Self::Failed,
        Self::Done,
    ];

    pub const fn permits_source_write(self) -> bool {
        matches!(self, Self::Done)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum WorkItemError {
    #[error("external work-item provider configuration is invalid")]
    InvalidConfiguration,
    #[error("external work-item snapshot is invalid")]
    InvalidSnapshot,
    #[error("external work-item provider is unsupported")]
    UnsupportedProvider,
    #[error("external work-item plugin failed: {0}")]
    Plugin(String),
    #[error("external work-item persistence failed: {0}")]
    Persistence(String),
    #[error("external work-item identity does not match configured provider")]
    ProviderIdentityMismatch,
    #[error("external work item `{0}` is already imported")]
    DuplicateImport(String),
    #[error("imported work item could not enter the task projections")]
    TaskProjection,
    #[error("external work-item capability is unsupported")]
    CapabilityRefused,
    #[error("external work item requires a confirmed workflow")]
    WorkflowRequired,
    #[error("external work item task is not Done")]
    NotDone,
    #[error("completion delivery failed")]
    DeliveryFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkItemPreview {
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedWorkItem {
    pub local_task: BoardTask,
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
}

#[derive(Clone, Debug, Default)]
pub struct WorkItemRegistry {
    configured: BTreeMap<WorkItemProviderId, WorkItemProviderConfig>,
    imported: BTreeMap<WorkItemIdentity, ImportedWorkItem>,
    board: BoardProjection,
    orchestrator: TaskOrchestrator,
}

impl WorkItemRegistry {
    pub fn configure(&mut self, config: WorkItemProviderConfig) {
        self.configured.insert(config.plugin_id.clone(), config);
    }

    pub fn select(
        &self,
        plugin_id: &WorkItemProviderId,
    ) -> Result<&WorkItemProviderConfig, WorkItemError> {
        self.configured
            .get(plugin_id)
            .ok_or(WorkItemError::UnsupportedProvider)
    }

    pub fn preview(
        &self,
        snapshot: WorkItemSnapshot,
        project_id: ProjectId,
        workflow_def_id: Option<Uuid>,
    ) -> Result<WorkItemPreview, WorkItemError> {
        snapshot.validate()?;
        let provider = self.select(&snapshot.identity.plugin_id)?;
        if provider.host != snapshot.identity.host || provider.project != snapshot.identity.project
        {
            return Err(WorkItemError::ProviderIdentityMismatch);
        }
        let task_id = TaskId::generate();
        Ok(WorkItemPreview {
            workflow: WorkflowSelection::default_for(task_id, project_id, workflow_def_id),
            snapshot,
        })
    }

    pub async fn preview_from_plugin(
        &self,
        process: &crate::plugin::PluginProcess,
        provider: &PluginWorkItemProvider,
        host: impl Into<String>,
        project: impl Into<String>,
        native_id: impl Into<String>,
        project_id: ProjectId,
        workflow_def_id: Option<Uuid>,
    ) -> Result<WorkItemPreview, WorkItemError> {
        let lookup = WorkItemLookup {
            plugin_id: provider.plugin_id.clone(),
            host: host.into(),
            project: project.into(),
            native_id: native_id.into(),
        };
        let snapshot = provider.normalize(snapshot_from_plugin(process, &lookup).await?)?;
        self.preview(snapshot, project_id, workflow_def_id)
    }

    pub async fn configure_persisted(
        &mut self,
        store: &crate::store::Store,
        config: WorkItemProviderConfig,
    ) -> Result<(), WorkItemError> {
        store
            .save_external_work_item_provider(&config)
            .await
            .map_err(|error| WorkItemError::Persistence(error.to_string()))?;
        self.configure(config);
        Ok(())
    }

    pub async fn load_persisted(
        &mut self,
        store: &crate::store::Store,
    ) -> Result<(), WorkItemError> {
        let configs = store
            .load_external_work_item_providers()
            .await
            .map_err(|error| WorkItemError::Persistence(error.to_string()))?;
        for config in configs {
            self.configure(config);
        }
        Ok(())
    }

    pub async fn persist_import(
        &self,
        store: &crate::store::Store,
        imported: &ImportedWorkItem,
    ) -> Result<bool, WorkItemError> {
        store
            .persist_external_work_item(imported.local_task.id, &imported.snapshot)
            .await
            .map_err(|error| WorkItemError::Persistence(error.to_string()))
    }

    pub fn import_confirmed(
        &mut self,
        preview: WorkItemPreview,
    ) -> Result<ImportedWorkItem, WorkItemError> {
        let identity = preview.snapshot.identity.clone();
        if self.imported.contains_key(&identity) {
            return Err(WorkItemError::DuplicateImport(
                identity.native_id.clone(),
            ));
        }
        if !preview.workflow.confirmed {
            return Err(WorkItemError::WorkflowRequired);
        }
        let mut task = BoardTask::new(
            preview.workflow.project_id,
            preview.workflow.task_id,
            preview.snapshot.title.clone(),
            None,
        );
        task.description = preview.snapshot.body.clone();
        task.external_work_item = Some(preview.snapshot.clone());
        let mut board = self.board.clone();
        board
            .apply(BoardEvent::Created {
                task: Box::new(task.clone()),
            })
            .map_err(|_| WorkItemError::TaskProjection)?;
        let mut orchestrator = self.orchestrator.clone();
        orchestrator
            .register(task.clone(), preview.workflow.clone())
            .map_err(|_| WorkItemError::TaskProjection)?;
        let imported = ImportedWorkItem {
            local_task: task,
            snapshot: preview.snapshot,
            workflow: preview.workflow,
        };
        self.board = board;
        self.orchestrator = orchestrator;
        self.imported.insert(identity, imported.clone());
        Ok(imported)
    }

    pub fn imported(&self, identity: &WorkItemIdentity) -> Option<&ImportedWorkItem> {
        self.imported.get(identity)
    }

    pub fn board(&self) -> &BoardProjection {
        &self.board
    }

    pub fn orchestrator(&self) -> &TaskOrchestrator {
        &self.orchestrator
    }

    pub fn providers(&self) -> impl Iterator<Item = &WorkItemProviderConfig> {
        self.configured.values()
    }

    /// Source-side edits are intentionally ignored after import.
    pub fn source_edit_does_not_sync(&self, identity: &WorkItemIdentity, title: &str) -> bool {
        self.imported(identity)
            .is_some_and(|item| item.local_task.summary != title)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompletionEvent {
    pub id: Uuid,
    pub task_id: TaskId,
    pub locator: String,
    pub evidence: Vec<ArtifactId>,
    pub comment: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionDelivery {
    pub event: CompletionEvent,
    pub attempts: u32,
    pub commented: bool,
    pub resolved: Option<bool>,
}

pub trait CompletionTransport {
    fn comment(&mut self, event: &CompletionEvent) -> Result<(), WorkItemError>;
    fn resolve(&mut self, event: &CompletionEvent) -> Result<(), WorkItemError>;
}

#[derive(Clone, Debug, Default)]
pub struct CompletionOutbox {
    deliveries: BTreeMap<TaskId, CompletionDelivery>,
}

impl CompletionOutbox {
    pub fn enqueue_done(
        &mut self,
        task: &BoardTask,
        evidence: Vec<ArtifactId>,
        supports_resolution: bool,
    ) -> Result<&CompletionEvent, WorkItemError> {
        self.enqueue_done_at_state(
            task,
            LocalWorkState::Done,
            evidence,
            supports_resolution,
        )
    }

    pub fn enqueue_done_at_state(
        &mut self,
        task: &BoardTask,
        state: LocalWorkState,
        evidence: Vec<ArtifactId>,
        supports_resolution: bool,
    ) -> Result<&CompletionEvent, WorkItemError> {
        if task.column != TaskColumn::Done || !state.permits_source_write() {
            return Err(WorkItemError::NotDone);
        }
        let entry = self.deliveries.entry(task.id).or_insert_with(|| {
            let locator = format!("locus://{}/task/{}", task.project_id, task.id);
            let comment = format!(
                "Completed {} with evidence: {}",
                locator,
                evidence
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            CompletionDelivery {
                event: CompletionEvent {
                    id: Uuid::new_v4(),
                    task_id: task.id,
                    locator,
                    evidence,
                    comment,
                },
                attempts: 0,
                commented: false,
                resolved: supports_resolution.then_some(false),
            }
        });
        Ok(&entry.event)
    }

    pub fn deliver(
        &mut self,
        task_id: TaskId,
        transport: &mut impl CompletionTransport,
        supports_resolution: bool,
    ) -> Result<(), WorkItemError> {
        let delivery = self
            .deliveries
            .get_mut(&task_id)
            .ok_or(WorkItemError::DeliveryFailed)?;
        if !delivery.commented {
            delivery.attempts += 1;
            transport.comment(&delivery.event)?;
            delivery.commented = true;
        }
        if supports_resolution && delivery.resolved != Some(true) {
            transport.resolve(&delivery.event)?;
            delivery.resolved = Some(true);
        }
        Ok(())
    }

    pub fn enqueue_done_with_provider(
        &mut self,
        task: &BoardTask,
        evidence: Vec<ArtifactId>,
        provider: &impl ExternalWorkItemProvider,
    ) -> Result<&CompletionEvent, WorkItemError> {
        let capabilities = provider.capabilities();
        if !capabilities.comments {
            return Err(WorkItemError::CapabilityRefused);
        }
        self.enqueue_done(task, evidence, capabilities.resolve)
    }

    pub fn deliver_with_provider(
        &mut self,
        task_id: TaskId,
        transport: &mut impl CompletionTransport,
        provider: &impl ExternalWorkItemProvider,
    ) -> Result<(), WorkItemError> {
        let capabilities = provider.capabilities();
        if !capabilities.comments {
            return Err(WorkItemError::CapabilityRefused);
        }
        self.deliver(task_id, transport, capabilities.resolve)
    }

    pub async fn deliver_via_plugin(
        &mut self,
        task_id: TaskId,
        process: &crate::plugin::PluginProcess,
        provider: &PluginWorkItemProvider,
        identity: &WorkItemIdentity,
    ) -> Result<(), WorkItemError> {
        let capabilities = provider.capabilities;
        if !capabilities.comments || identity.plugin_id != provider.plugin_id {
            return Err(WorkItemError::CapabilityRefused);
        }

        let (event, should_comment, should_resolve) = {
            let delivery = self
                .deliveries
                .get_mut(&task_id)
                .ok_or(WorkItemError::DeliveryFailed)?;
            if !delivery.commented {
                delivery.attempts += 1;
            }
            (
                delivery.event.clone(),
                !delivery.commented,
                capabilities.resolve && delivery.resolved != Some(true),
            )
        };
        let params = serde_json::json!({ "identity": identity, "event": event });
        if should_comment {
            process
                .call(WORK_ITEM_COMMENT_METHOD, params.clone())
                .await
                .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
            self.deliveries
                .get_mut(&task_id)
                .ok_or(WorkItemError::DeliveryFailed)?
                .commented = true;
        }
        if should_resolve {
            process
                .call(WORK_ITEM_RESOLVE_METHOD, params)
                .await
                .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
            self.deliveries
                .get_mut(&task_id)
                .ok_or(WorkItemError::DeliveryFailed)?
                .resolved = Some(true);
        }
        Ok(())
    }

    pub async fn persist_delivery(
        &self,
        store: &crate::store::Store,
        task_id: TaskId,
        snapshot: &WorkItemSnapshot,
    ) -> Result<bool, WorkItemError> {
        let delivery = self
            .delivery(task_id)
            .ok_or(WorkItemError::DeliveryFailed)?;
        store
            .enqueue_external_completion(delivery, snapshot)
            .await
            .map_err(|error| WorkItemError::Persistence(error.to_string()))
    }

    pub fn delivery(&self, task_id: TaskId) -> Option<&CompletionDelivery> {
        self.deliveries.get(&task_id)
    }
}

#[cfg(test)]
mod work_item {
    use super::*;

    fn config(plugin_id: &str) -> WorkItemProviderConfig {
        WorkItemProviderConfig::new(plugin_id, "provider.example", "org/repo").unwrap()
    }

    fn provider(plugin_id: &str, resolve: bool) -> PluginWorkItemProvider {
        PluginWorkItemProvider::new(
            plugin_id,
            WorkItemCapabilities {
                comments: true,
                resolve,
            },
        )
        .unwrap()
    }

    fn snapshot(plugin_id: &str) -> WorkItemSnapshot {
        WorkItemSnapshot {
            identity: WorkItemIdentity {
                plugin_id: WorkItemProviderId::new(plugin_id).unwrap(),
                host: "provider.example".into(),
                project: "org/repo".into(),
                native_id: "42".into(),
            },
            url: "https://provider.example/item/42".into(),
            title: "Imported issue".into(),
            body: "Body".into(),
            labels: vec!["bug".into()],
            status: "open".into(),
        }
    }

    fn registry() -> WorkItemRegistry {
        let mut registry = WorkItemRegistry::default();
        registry.configure(config("fixture.provider"));
        registry
    }

    #[test]
    fn schema() {
        let migration = include_str!("../../../migrations/0023_external_work_items.up.sql");
        assert!(migration.contains("board.external_work_items"));
        assert!(migration.contains("board.external_completion_outbox"));
    }

    #[test]
    fn contract_types() {
        let plugin_id = WorkItemProviderId::new("user.tracker").unwrap();
        assert_eq!(plugin_id.as_str(), "user.tracker");
        assert!(provider("user.tracker", true).capabilities.comments);
    }

    #[test]
    fn provider_configuration() {
        assert_eq!(config("fixture.provider").project, "org/repo");
        assert_eq!(config("fixture.provider").plugin_id.as_str(), "fixture.provider");
        assert_eq!(registry().providers().count(), 1);
    }

    #[test]
    fn adapter_selection() {
        let registry = registry();
        let configured = WorkItemProviderId::new("fixture.provider").unwrap();
        let missing = WorkItemProviderId::new("missing.provider").unwrap();
        assert!(registry.select(&configured).is_ok());
        assert!(registry.select(&missing).is_err());
    }

    #[test]
    fn plugin_adapter_bridge() {
        let adapter = provider("fixture.provider", true);
        let normalized = adapter.normalize(snapshot("fixture.provider")).unwrap();
        assert_eq!(normalized.identity.plugin_id.as_str(), "fixture.provider");
        assert!(adapter.capabilities.resolve);
    }

    #[test]
    fn plugin_snapshot_contract() {
        let snapshot = snapshot("fixture.jira");
        assert_eq!(snapshot.identity.plugin_id.as_str(), "fixture.jira");
        assert_eq!(snapshot.identity.native_id, "42");
        assert_eq!(snapshot.labels, vec!["bug"]);
        assert_eq!(snapshot.status, "open");
    }

    #[test]
    fn preview() {
        let preview = registry()
            .preview(
                snapshot("fixture.provider"),
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        assert!(preview.snapshot.title.contains("Imported"));
    }

    #[test]
    fn import_creates_task() {
        let mut registry = registry();
        let project_id = ProjectId::generate();
        let mut preview = registry
            .preview(
                snapshot("fixture.provider"),
                project_id,
                Some(Uuid::new_v4()),
            )
            .unwrap();
        preview.workflow = preview.workflow.confirm().unwrap();
        let task_id = preview.workflow.task_id;
        let imported = registry.import_confirmed(preview).expect("import");
        assert_eq!(registry.board().task(task_id).unwrap().id, task_id);
        assert_eq!(
            registry
                .orchestrator()
                .detail(task_id)
                .unwrap()
                .workflow_def_id,
            imported.workflow.workflow_def_id
        );
        assert_eq!(
            imported
                .local_task
                .external_work_item
                .unwrap()
                .identity
                .native_id,
            "42"
        );
    }

    #[test]
    fn opaque_native_identity_is_preserved() {
        let mut registry = WorkItemRegistry::default();
        registry.configure(config("fixture.tracker"));
        let mut imported_snapshot = snapshot("fixture.tracker");
        imported_snapshot.identity.native_id = "TRACK-42".into();
        imported_snapshot.url = "https://provider.example/item/TRACK-42".into();
        let mut preview = registry
            .preview(
                imported_snapshot,
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        preview.workflow = preview.workflow.confirm().unwrap();
        let imported = registry.import_confirmed(preview).unwrap();
        assert!(imported.local_task.external_issue.is_none());
        assert_eq!(imported.snapshot.identity.native_id, "TRACK-42");
    }

    #[test]
    fn deduplicates_import() {
        let mut registry = registry();
        let mut preview = registry
            .preview(
                snapshot("fixture.provider"),
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        preview.workflow = preview.workflow.confirm().unwrap();
        let identity = preview.snapshot.identity.clone();
        registry.import_confirmed(preview).unwrap();
        let mut duplicate = registry
            .preview(
                snapshot("fixture.provider"),
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        duplicate.snapshot.identity = identity;
        duplicate.workflow = duplicate.workflow.confirm().unwrap();
        assert!(matches!(
            registry.import_confirmed(duplicate),
            Err(WorkItemError::DuplicateImport(_))
        ));
    }

    #[test]
    fn no_source_sync() {
        let mut registry = registry();
        let mut preview = registry
            .preview(
                snapshot("fixture.provider"),
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        let identity = preview.snapshot.identity.clone();
        preview.workflow = preview.workflow.confirm().unwrap();
        registry.import_confirmed(preview).unwrap();
        assert!(registry.source_edit_does_not_sync(&identity, "edited upstream"));
    }

    #[test]
    fn no_write_before_done() {
        let task = done_task();
        for state in LocalWorkState::ALL {
            let mut outbox = CompletionOutbox::default();
            assert_eq!(
                outbox
                    .enqueue_done_at_state(&task, state, vec![], true)
                    .is_ok(),
                state == LocalWorkState::Done
            );
        }
        assert!(LocalWorkState::Done.permits_source_write());
    }

    #[derive(Default)]
    struct Transport {
        comments: usize,
        resolves: usize,
        fail_once: bool,
    }
    impl CompletionTransport for Transport {
        fn comment(&mut self, _event: &CompletionEvent) -> Result<(), WorkItemError> {
            if self.fail_once {
                self.fail_once = false;
                return Err(WorkItemError::DeliveryFailed);
            }
            self.comments += 1;
            Ok(())
        }
        fn resolve(&mut self, _event: &CompletionEvent) -> Result<(), WorkItemError> {
            self.resolves += 1;
            Ok(())
        }
    }

    fn done_task() -> BoardTask {
        let mut task = BoardTask::new(ProjectId::generate(), TaskId::generate(), "done", None);
        task.column = TaskColumn::Done;
        task
    }

    #[test]
    fn completion_outbox() {
        let task = done_task();
        let mut outbox = CompletionOutbox::default();
        let first = outbox
            .enqueue_done(&task, vec![ArtifactId::generate()], true)
            .unwrap()
            .id;
        let second = outbox.enqueue_done(&task, vec![], true).unwrap().id;
        assert_eq!(first, second);
    }

    #[test]
    fn completion_comment() {
        let task = done_task();
        let mut outbox = CompletionOutbox::default();
        let event = outbox
            .enqueue_done(&task, vec![ArtifactId::generate()], false)
            .unwrap();
        assert!(event.comment.contains("locus://"));
        assert!(event.comment.contains("evidence"));
    }

    #[test]
    fn completion_resolves() {
        let task = done_task();
        let mut outbox = CompletionOutbox::default();
        outbox.enqueue_done(&task, vec![], true).unwrap();
        let mut transport = Transport::default();
        outbox.deliver(task.id, &mut transport, true).unwrap();
        assert_eq!(transport.resolves, 1);
    }

    #[test]
    fn resolution_capability_refused() {
        let task = done_task();
        let mut outbox = CompletionOutbox::default();
        outbox.enqueue_done(&task, vec![], false).unwrap();
        let mut transport = Transport::default();
        outbox.deliver(task.id, &mut transport, false).unwrap();
        assert_eq!(transport.resolves, 0);
    }

    #[test]
    fn completion_retry_is_one_way() {
        let task = done_task();
        let mut outbox = CompletionOutbox::default();
        outbox.enqueue_done(&task, vec![], false).unwrap();
        let mut transport = Transport {
            fail_once: true,
            ..Default::default()
        };
        assert!(outbox.deliver(task.id, &mut transport, false).is_err());
        outbox.deliver(task.id, &mut transport, false).unwrap();
        assert_eq!(transport.comments, 1);
    }

    #[test]
    fn provider_conformance() {
        for plugin_id in ["fixture.github", "fixture.gitlab", "fixture.jira", "user.tracker"] {
            let provider = provider(plugin_id, true);
            assert_eq!(
                provider
                    .normalize(snapshot(plugin_id))
                    .unwrap()
                    .identity
                    .plugin_id
                    .as_str(),
                plugin_id
            );
        }
    }

    #[test]
    fn lookup_round_trip() {
        let snapshot = snapshot("fixture.provider");
        let lookup = WorkItemLookup::from(&snapshot.identity);
        assert_eq!(lookup.plugin_id, snapshot.identity.plugin_id);
        assert_eq!(lookup.native_id, "42");
    }
}
