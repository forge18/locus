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
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct WorkItemProviderId(String);

impl WorkItemProviderId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkItemError> {
        let value = value.into();
        if value.trim().is_empty() || value.contains('\0') {
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkItemProviderKey {
    plugin_id: WorkItemProviderId,
    host: String,
    project: String,
}

impl WorkItemProviderKey {
    fn new(plugin_id: &WorkItemProviderId, host: &str, project: &str) -> Self {
        Self {
            plugin_id: plugin_id.clone(),
            host: host.into(),
            project: project.into(),
        }
    }
}

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
        if config.host.trim().is_empty()
            || config.project.trim().is_empty()
            || config.host.contains('\0')
            || config.project.contains('\0')
        {
            return Err(WorkItemError::InvalidConfiguration);
        }
        Ok(config)
    }

    fn key(&self) -> WorkItemProviderKey {
        WorkItemProviderKey::new(&self.plugin_id, &self.host, &self.project)
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
        let valid_url = Url::parse(&self.url).is_ok_and(|url| {
            url.scheme() == "https"
                && url
                    .host_str()
                    .is_some_and(|host| host.eq_ignore_ascii_case(&self.identity.host))
                && url.port().is_none_or(|port| port == 443)
                && url.username().is_empty()
                && url.password().is_none()
        });
        if self.identity.plugin_id.as_str().trim().is_empty()
            || self.identity.host.trim().is_empty()
            || self.identity.project.trim().is_empty()
            || self.identity.native_id.trim().is_empty()
            || !valid_url
            || self.title.trim().is_empty()
            || self.status.trim().is_empty()
            || self
                .labels
                .iter()
                .any(|label| label.trim().is_empty() || label.contains('\0'))
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
    #[error("external work-item provider has multiple configured instances")]
    AmbiguousProvider(WorkItemProviderId),
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemPreview {
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedWorkItem {
    pub local_task: BoardTask,
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
}

#[derive(Clone, Debug, Default)]
pub struct WorkItemRegistry {
    configured: BTreeMap<WorkItemProviderKey, WorkItemProviderConfig>,
    imported: BTreeMap<WorkItemIdentity, ImportedWorkItem>,
    board: BoardProjection,
    orchestrator: TaskOrchestrator,
}

impl WorkItemRegistry {
    pub fn configure(&mut self, config: WorkItemProviderConfig) {
        self.configured.insert(config.key(), config);
    }

    pub fn select(
        &self,
        plugin_id: &WorkItemProviderId,
    ) -> Result<&WorkItemProviderConfig, WorkItemError> {
        let mut matches = self
            .configured
            .values()
            .filter(|config| &config.plugin_id == plugin_id);
        match (matches.next(), matches.next()) {
            (Some(config), None) => Ok(config),
            (Some(_), Some(_)) => Err(WorkItemError::AmbiguousProvider(plugin_id.clone())),
            (None, _) => Err(WorkItemError::UnsupportedProvider),
        }
    }

    pub fn select_for(
        &self,
        identity: &WorkItemIdentity,
    ) -> Result<&WorkItemProviderConfig, WorkItemError> {
        if let Some(config) = self.configured.get(&WorkItemProviderKey::new(
            &identity.plugin_id,
            &identity.host,
            &identity.project,
        )) {
            return Ok(config);
        }
        if self
            .configured
            .values()
            .any(|config| config.plugin_id == identity.plugin_id)
        {
            return Err(WorkItemError::ProviderIdentityMismatch);
        }
        Err(WorkItemError::UnsupportedProvider)
    }

    pub fn preview(
        &self,
        snapshot: WorkItemSnapshot,
        project_id: ProjectId,
        workflow_def_id: Option<Uuid>,
    ) -> Result<WorkItemPreview, WorkItemError> {
        snapshot.validate()?;
        self.select_for(&snapshot.identity)?;
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
        lookup: WorkItemLookup,
        project_id: ProjectId,
        workflow_def_id: Option<Uuid>,
    ) -> Result<WorkItemPreview, WorkItemError> {
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
            .persist_imported_task(&imported.local_task, &imported.snapshot, &imported.workflow)
            .await
            .map_err(|error| WorkItemError::Persistence(error.to_string()))
    }

    pub fn restore_imported(
        &mut self,
        task: BoardTask,
        snapshot: WorkItemSnapshot,
        workflow: WorkflowSelection,
    ) -> Result<(), WorkItemError> {
        self.restore_imported_with_state(task, snapshot, workflow, Vec::new(), Vec::new())
    }

    pub fn restore_imported_with_state(
        &mut self,
        task: BoardTask,
        snapshot: WorkItemSnapshot,
        workflow: WorkflowSelection,
        runs: Vec<crate::services::task::TaskRunLink>,
        evidence: Vec<crate::services::task::TaskEvidenceLink>,
    ) -> Result<(), WorkItemError> {
        snapshot.validate()?;
        if self.imported.contains_key(&snapshot.identity) {
            return Err(WorkItemError::DuplicateImport(
                snapshot.identity.native_id.clone(),
            ));
        }
        if task.external_work_item.as_ref() != Some(&snapshot) {
            return Err(WorkItemError::ProviderIdentityMismatch);
        }
        if !workflow.confirmed
            || workflow.task_id != task.id
            || workflow.project_id != task.project_id
            || workflow.workflow_def_id.is_none()
        {
            return Err(WorkItemError::WorkflowRequired);
        }
        let mut board = self.board.clone();
        board
            .apply(BoardEvent::Created {
                task: Box::new(task.clone()),
            })
            .map_err(|_| WorkItemError::TaskProjection)?;
        let mut orchestrator = self.orchestrator.clone();
        let root_session_id = task
            .session_id
            .or_else(|| runs.first().map(|run| run.root_session_id));
        orchestrator
            .restore_task_state(
                task.clone(),
                workflow.clone(),
                root_session_id,
                runs,
                evidence,
                Some(snapshot.url.clone()),
            )
            .map_err(|_| WorkItemError::TaskProjection)?;
        self.board = board;
        self.orchestrator = orchestrator;
        self.imported.insert(
            snapshot.identity.clone(),
            ImportedWorkItem {
                local_task: task,
                snapshot,
                workflow,
            },
        );
        Ok(())
    }

    pub fn move_to_done(
        &mut self,
        task_id: TaskId,
        actor: crate::services::board::BoardActor,
        evidence: Vec<crate::services::board::BoardEvidenceLink>,
    ) -> Result<BoardTask, WorkItemError> {
        let task = self
            .board
            .task(task_id)
            .cloned()
            .ok_or(WorkItemError::TaskProjection)?;
        let identity = task
            .external_work_item
            .as_ref()
            .map(|snapshot| snapshot.identity.clone())
            .ok_or(WorkItemError::TaskProjection)?;
        let event = task
            .transition(TaskColumn::Done, actor, evidence)
            .map_err(|_| WorkItemError::TaskProjection)?;
        let mut board = self.board.clone();
        board
            .apply(event)
            .map_err(|_| WorkItemError::TaskProjection)?;
        let updated = board
            .task(task_id)
            .cloned()
            .ok_or(WorkItemError::TaskProjection)?;
        let mut orchestrator = self.orchestrator.clone();
        orchestrator
            .update_task(updated.clone())
            .map_err(|_| WorkItemError::TaskProjection)?;
        self.board = board;
        self.orchestrator = orchestrator;
        let imported = self
            .imported
            .get_mut(&identity)
            .ok_or(WorkItemError::TaskProjection)?;
        imported.local_task = updated.clone();
        Ok(updated)
    }

    pub fn import_confirmed(
        &mut self,
        preview: WorkItemPreview,
    ) -> Result<ImportedWorkItem, WorkItemError> {
        preview.snapshot.validate()?;
        let identity = preview.snapshot.identity.clone();
        self.select_for(&identity)?;
        if self.imported.contains_key(&identity) {
            return Err(WorkItemError::DuplicateImport(identity.native_id.clone()));
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

    pub fn imported_tasks(&self) -> impl Iterator<Item = &ImportedWorkItem> {
        self.imported.values()
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
    fn enqueue_done_at_state(
        &mut self,
        task: &BoardTask,
        state: LocalWorkState,
        evidence: Vec<ArtifactId>,
        capabilities: WorkItemCapabilities,
    ) -> Result<&CompletionEvent, WorkItemError> {
        if task.column != TaskColumn::Done || !state.permits_source_write() {
            return Err(WorkItemError::NotDone);
        }
        if !capabilities.comments {
            return Err(WorkItemError::CapabilityRefused);
        }
        let entry = self.deliveries.entry(task.id).or_insert_with(|| {
            let locator = format!("locus://{}/task/{}", task.project_id, task.id);
            let id = Uuid::new_v4();
            let comment = format!(
                "Completed {} with evidence: {}\n<!-- locus-completion:{} -->",
                locator,
                evidence
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                id
            );
            CompletionDelivery {
                event: CompletionEvent {
                    id,
                    task_id: task.id,
                    locator,
                    evidence,
                    comment,
                },
                attempts: 0,
                commented: false,
                resolved: capabilities.resolve.then_some(false),
            }
        });
        Ok(&entry.event)
    }

    pub fn enqueue_done_with_provider(
        &mut self,
        task: &BoardTask,
        evidence: Vec<ArtifactId>,
        provider: &impl ExternalWorkItemProvider,
    ) -> Result<&CompletionEvent, WorkItemError> {
        self.enqueue_done_at_state(
            task,
            LocalWorkState::Done,
            evidence,
            provider.capabilities(),
        )
    }

    fn deliver(
        &mut self,
        task_id: TaskId,
        transport: &mut impl CompletionTransport,
        capabilities: WorkItemCapabilities,
    ) -> Result<(), WorkItemError> {
        if !capabilities.comments {
            return Err(WorkItemError::CapabilityRefused);
        }
        let delivery = self
            .deliveries
            .get_mut(&task_id)
            .ok_or(WorkItemError::DeliveryFailed)?;
        if !delivery.commented || (capabilities.resolve && delivery.resolved != Some(true)) {
            delivery.attempts = delivery.attempts.saturating_add(1);
        }
        if !delivery.commented {
            transport.comment(&delivery.event)?;
            delivery.commented = true;
        }
        if capabilities.resolve && delivery.resolved != Some(true) {
            transport.resolve(&delivery.event)?;
            delivery.resolved = Some(true);
        }
        Ok(())
    }

    pub fn deliver_with_provider(
        &mut self,
        task_id: TaskId,
        transport: &mut impl CompletionTransport,
        provider: &impl ExternalWorkItemProvider,
    ) -> Result<(), WorkItemError> {
        self.deliver(task_id, transport, provider.capabilities())
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
            if !delivery.commented || (capabilities.resolve && delivery.resolved != Some(true)) {
                delivery.attempts = delivery.attempts.saturating_add(1);
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

    pub fn record_delivery_failure(
        &mut self,
        task_id: TaskId,
    ) -> Result<CompletionDelivery, WorkItemError> {
        let delivery = self
            .deliveries
            .get_mut(&task_id)
            .ok_or(WorkItemError::DeliveryFailed)?;
        delivery.attempts = delivery.attempts.saturating_add(1);
        Ok(delivery.clone())
    }

    pub fn restore_delivery(&mut self, delivery: CompletionDelivery) -> Result<(), WorkItemError> {
        if let Some(existing) = self.deliveries.get(&delivery.event.task_id) {
            return if existing.event == delivery.event {
                Ok(())
            } else {
                Err(WorkItemError::DeliveryFailed)
            };
        }
        self.deliveries.insert(delivery.event.task_id, delivery);
        Ok(())
    }

    pub fn delivery(&self, task_id: TaskId) -> Option<&CompletionDelivery> {
        self.deliveries.get(&task_id)
    }
}

#[cfg(test)]
mod work_item {
    use super::*;

    fn config(plugin_id: &str) -> WorkItemProviderConfig {
        config_at(plugin_id, "provider.example", "org/repo")
    }

    fn config_at(plugin_id: &str, host: &str, project: &str) -> WorkItemProviderConfig {
        WorkItemProviderConfig::new(plugin_id, host, project).unwrap()
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
        assert!(migration.contains("plugin_id"));
        assert!(!migration.contains("resolution_supported"));
        let upgrade =
            include_str!("../../../migrations/0024_external_work_item_provider_instances.up.sql");
        assert!(upgrade.contains("workflow_def_id"));
        assert!(upgrade.contains("locator"));
        assert!(upgrade.contains("resolution_supported"));
        assert!(!migration.contains("provider_kind"));
    }

    #[test]
    fn contract_types() {
        let plugin_id = WorkItemProviderId::new("user.tracker").unwrap();
        assert_eq!(plugin_id.as_str(), "user.tracker");
        assert!(provider("user.tracker", true).capabilities.comments);
    }

    #[test]
    fn snapshot_url_requires_https_without_credentials() {
        let mut snapshot = snapshot("fixture.provider");
        snapshot.url = "javascript:alert(1)".into();
        assert_eq!(snapshot.validate(), Err(WorkItemError::InvalidSnapshot));
        snapshot.url = "https://user:pass@provider.example/item/42".into();
        assert_eq!(snapshot.validate(), Err(WorkItemError::InvalidSnapshot));
        snapshot.url = "https://attacker.example/item/42".into();
        assert_eq!(snapshot.validate(), Err(WorkItemError::InvalidSnapshot));
        snapshot.url = "https://provider.example/item/42".into();
        snapshot.status.clear();
        assert_eq!(snapshot.validate(), Err(WorkItemError::InvalidSnapshot));
    }

    #[test]
    fn provider_configuration() {
        assert_eq!(config("fixture.provider").project, "org/repo");
        assert_eq!(
            config("fixture.provider").plugin_id.as_str(),
            "fixture.provider"
        );
        assert_eq!(registry().providers().count(), 1);
    }

    #[test]
    fn provider_instances_are_keyed_by_identity() {
        let mut registry = WorkItemRegistry::default();
        registry.configure(config_at("github", "github.com", "org/one"));
        registry.configure(config_at("github", "github.com", "org/two"));
        assert_eq!(registry.providers().count(), 2);
        assert!(matches!(
            registry.select(&WorkItemProviderId::new("github").unwrap()),
            Err(WorkItemError::AmbiguousProvider(_))
        ));
        let identity = WorkItemIdentity {
            plugin_id: WorkItemProviderId::new("github").unwrap(),
            host: "github.com".into(),
            project: "org/two".into(),
            native_id: "42".into(),
        };
        assert_eq!(registry.select_for(&identity).unwrap().project, "org/two");
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
    fn restore_imported_preserves_confirmed_workflow() {
        let mut source = registry();
        let project_id = ProjectId::generate();
        let mut preview = source
            .preview(
                snapshot("fixture.provider"),
                project_id,
                Some(Uuid::new_v4()),
            )
            .unwrap();
        preview.workflow = preview.workflow.confirm().unwrap();
        let imported = source.import_confirmed(preview).unwrap();

        let mut restored = registry();
        restored
            .restore_imported(
                imported.local_task,
                imported.snapshot,
                imported.workflow.clone(),
            )
            .unwrap();
        let detail = restored
            .orchestrator()
            .detail(imported.workflow.task_id)
            .unwrap();
        assert_eq!(detail.workflow_def_id, imported.workflow.workflow_def_id);
    }

    #[test]
    fn restore_survives_unconfigured_provider_instance() {
        let mut source = registry();
        let mut preview = source
            .preview(
                snapshot("fixture.provider"),
                ProjectId::generate(),
                Some(Uuid::new_v4()),
            )
            .unwrap();
        preview.workflow = preview.workflow.confirm().unwrap();
        let imported = source.import_confirmed(preview).unwrap();

        let mut restored = WorkItemRegistry::default();
        restored
            .restore_imported(imported.local_task, imported.snapshot, imported.workflow)
            .unwrap();
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
        let capabilities = WorkItemCapabilities {
            comments: true,
            resolve: true,
        };
        for state in LocalWorkState::ALL {
            let mut outbox = CompletionOutbox::default();
            assert_eq!(
                outbox
                    .enqueue_done_at_state(&task, state, vec![], capabilities)
                    .is_ok(),
                state == LocalWorkState::Done
            );
        }
        assert!(LocalWorkState::Done.permits_source_write());
    }

    #[test]
    fn completion_requires_actual_done_column() {
        let provider = provider("fixture.provider", true);
        for column in TaskColumn::ALL {
            let mut task = done_task();
            task.column = column;
            let mut outbox = CompletionOutbox::default();
            assert_eq!(
                outbox
                    .enqueue_done_with_provider(&task, vec![], &provider)
                    .is_ok(),
                column == TaskColumn::Done
            );
        }
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
        let completion_provider = provider("fixture.provider", true);
        let mut outbox = CompletionOutbox::default();
        let first = outbox
            .enqueue_done_with_provider(&task, vec![ArtifactId::generate()], &completion_provider)
            .unwrap()
            .id;
        let second = outbox
            .enqueue_done_with_provider(&task, vec![], &completion_provider)
            .unwrap()
            .id;
        assert_eq!(first, second);
    }

    #[test]
    fn completion_restore_is_idempotent() {
        let task = done_task();
        let completion_provider = provider("fixture.provider", false);
        let mut original = CompletionOutbox::default();
        original
            .enqueue_done_with_provider(&task, vec![], &completion_provider)
            .unwrap();
        let delivery = original.delivery(task.id).unwrap().clone();
        let mut restored = CompletionOutbox::default();
        restored.restore_delivery(delivery.clone()).unwrap();
        restored.restore_delivery(delivery).unwrap();
        assert_eq!(
            restored.delivery(task.id).unwrap().event.id,
            original.delivery(task.id).unwrap().event.id
        );
    }

    #[test]
    fn completion_comment() {
        let task = done_task();
        let completion_provider = provider("fixture.provider", false);
        let mut outbox = CompletionOutbox::default();
        let event = outbox
            .enqueue_done_with_provider(&task, vec![ArtifactId::generate()], &completion_provider)
            .unwrap();
        assert!(event.comment.contains("locus://"));
        assert!(event.comment.contains("evidence"));
    }

    #[test]
    fn completion_resolves() {
        let task = done_task();
        let completion_provider = provider("fixture.provider", true);
        let mut outbox = CompletionOutbox::default();
        outbox
            .enqueue_done_with_provider(&task, vec![], &completion_provider)
            .unwrap();
        let mut transport = Transport::default();
        outbox
            .deliver_with_provider(task.id, &mut transport, &completion_provider)
            .unwrap();
        assert_eq!(transport.resolves, 1);
    }

    #[test]
    fn resolution_capability_refused() {
        let task = done_task();
        let completion_provider = provider("fixture.provider", false);
        let mut outbox = CompletionOutbox::default();
        outbox
            .enqueue_done_with_provider(&task, vec![], &completion_provider)
            .unwrap();
        let mut transport = Transport::default();
        outbox
            .deliver_with_provider(task.id, &mut transport, &completion_provider)
            .unwrap();
        assert_eq!(transport.resolves, 0);
    }

    #[test]
    fn completion_retry_is_one_way() {
        let task = done_task();
        let completion_provider = provider("fixture.provider", false);
        let mut outbox = CompletionOutbox::default();
        outbox
            .enqueue_done_with_provider(&task, vec![], &completion_provider)
            .unwrap();
        let mut transport = Transport {
            fail_once: true,
            ..Default::default()
        };
        assert!(outbox
            .deliver_with_provider(task.id, &mut transport, &completion_provider)
            .is_err());
        outbox
            .deliver_with_provider(task.id, &mut transport, &completion_provider)
            .unwrap();
        assert_eq!(transport.comments, 1);
    }

    #[test]
    fn provider_conformance() {
        for plugin_id in [
            "fixture.github",
            "fixture.gitlab",
            "fixture.jira",
            "user.tracker",
        ] {
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
