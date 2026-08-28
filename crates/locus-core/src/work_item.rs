//! Provider-neutral external work-item import, synchronization, and completion delivery.
//!
//! Import snapshots become local task state. Sync-capable providers exchange normalized
//! statuses and notes through the provider port; completion remains durable and idempotent.

use std::collections::BTreeMap;

use crate::{
    ids::{ArtifactId, ProjectId, TaskId},
    services::{
        board::{
            BoardActor, BoardComment, BoardCommentOrigin, BoardEvent, BoardEvidenceLink,
            BoardExternalEvidence, BoardProjection, BoardTask,
        },
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

/// The provider-owned translation between its remote status vocabulary and the fixed board.
/// `None` is intentional: an external status can be visible without being guessed into a
/// local column.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemStatusVocabulary {
    pub external_to_local: BTreeMap<String, Option<TaskColumn>>,
    pub local_to_external: BTreeMap<TaskColumn, String>,
    #[serde(default)]
    pub blocked_to_external: Option<String>,
}

impl WorkItemStatusVocabulary {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        if self.external_to_local.iter().any(|(status, column)| {
            status.trim().is_empty()
                || status.contains('\0')
                || column.is_some_and(|column| !TaskColumn::ALL.contains(&column))
        }) || self.local_to_external.iter().any(|(column, status)| {
            !TaskColumn::ALL.contains(column) || status.trim().is_empty() || status.contains('\0')
        }) || self
            .blocked_to_external
            .as_deref()
            .is_some_and(|status| status.trim().is_empty() || status.contains('\0'))
        {
            return Err(WorkItemError::InvalidSyncCapability);
        }
        Ok(())
    }

    pub fn local_status(&self, column: TaskColumn, blocked: bool) -> Result<&str, WorkItemError> {
        if blocked {
            if let Some(status) = self.blocked_to_external.as_deref() {
                return Ok(status);
            }
        }
        self.local_to_external
            .get(&column)
            .map(String::as_str)
            .ok_or(WorkItemError::UnmappedLocalStatus(column))
    }

    pub fn external_status(&self, status: &str) -> Option<TaskColumn> {
        self.external_to_local.get(status).copied().flatten()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemSyncCapability {
    pub vocabulary: WorkItemStatusVocabulary,
}

impl WorkItemSyncCapability {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        self.vocabulary.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemPullRequest {
    pub identity: WorkItemIdentity,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkItemPullChange {
    Status {
        id: String,
        status: String,
        occurred_at: String,
        author: String,
    },
    Note {
        id: String,
        body: String,
        occurred_at: String,
        author: String,
    },
}

impl WorkItemPullChange {
    pub fn id(&self) -> &str {
        match self {
            Self::Status { id, .. } | Self::Note { id, .. } => id,
        }
    }

    pub fn occurred_at(&self) -> &str {
        match self {
            Self::Status { occurred_at, .. } | Self::Note { occurred_at, .. } => occurred_at,
        }
    }

    pub fn validate(&self) -> Result<(), WorkItemError> {
        let (id, author) = match self {
            Self::Status {
                id, status, author, ..
            } => {
                if status.trim().is_empty() || status.contains('\0') {
                    return Err(WorkItemError::InvalidSyncChange);
                }
                (id, author)
            }
            Self::Note {
                id, body, author, ..
            } => {
                if body.trim().is_empty() || body.contains('\0') {
                    return Err(WorkItemError::InvalidSyncChange);
                }
                (id, author)
            }
        };
        if id.trim().is_empty()
            || author.trim().is_empty()
            || id.contains('\0')
            || author.contains('\0')
            || self.occurred_at().contains('\0')
        {
            return Err(WorkItemError::InvalidSyncChange);
        }
        time::OffsetDateTime::parse(
            self.occurred_at(),
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| WorkItemError::InvalidSyncChange)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemPullResult {
    pub next_cursor: Option<String>,
    pub changes: Vec<WorkItemPullChange>,
}

impl WorkItemPullResult {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        if self
            .next_cursor
            .as_deref()
            .is_some_and(|cursor| cursor.trim().is_empty() || cursor.contains('\0'))
        {
            return Err(WorkItemError::InvalidSyncChange);
        }
        self.changes
            .iter()
            .try_for_each(WorkItemPullChange::validate)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemPushStatusRequest {
    pub identity: WorkItemIdentity,
    pub column: TaskColumn,
    pub blocked: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemNote {
    pub id: String,
    pub body: String,
    pub author: String,
    pub occurred_at: String,
}

impl WorkItemNote {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        if self.id.trim().is_empty()
            || self.body.trim().is_empty()
            || self.author.trim().is_empty()
            || self.id.contains('\0')
            || self.body.contains('\0')
            || self.author.contains('\0')
        {
            return Err(WorkItemError::InvalidSyncChange);
        }
        time::OffsetDateTime::parse(
            &self.occurred_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| WorkItemError::InvalidSyncChange)?;
        Ok(())
    }
}

pub const LOCUS_NOTE_MARKER_PREFIX: &str = "<!-- locus-note:";

pub fn locus_note_marker(note_id: &str) -> String {
    format!("{LOCUS_NOTE_MARKER_PREFIX}{note_id} -->")
}

pub fn note_body_with_locus_marker(body: &str, note_id: &str) -> String {
    format!("{body}\n{}", locus_note_marker(note_id))
}

pub fn locus_note_marker_id(body: &str) -> Option<&str> {
    let start = body.find(LOCUS_NOTE_MARKER_PREFIX)? + LOCUS_NOTE_MARKER_PREFIX.len();
    let rest = &body[start..];
    let end = rest.find(" -->")?;
    (!rest[..end].is_empty()).then_some(&rest[..end])
}

fn parse_sync_timestamp(value: &str) -> Result<time::OffsetDateTime, WorkItemError> {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|_| WorkItemError::InvalidSyncChange)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemPushNoteRequest {
    pub identity: WorkItemIdentity,
    pub note: WorkItemNote,
}

pub const WORK_ITEM_SNAPSHOT_METHOD: &str = "work_item.snapshot";
pub const WORK_ITEM_COMMENT_METHOD: &str = "work_item.comment";
pub const WORK_ITEM_RESOLVE_METHOD: &str = "work_item.resolve";
pub const WORK_ITEM_SYNC_CAPABILITY_METHOD: &str = "work_item.sync_capability";
pub const WORK_ITEM_PULL_METHOD: &str = "work_item.pull";
pub const WORK_ITEM_PUSH_STATUS_METHOD: &str = "work_item.push_status";
pub const WORK_ITEM_PUSH_NOTE_METHOD: &str = "work_item.push_note";
pub const WORK_ITEM_SNAPSHOT_CAPABILITY: &str = "work_item.snapshot";
pub const WORK_ITEM_COMMENT_CAPABILITY: &str = "work_item.comment";
pub const WORK_ITEM_RESOLVE_CAPABILITY: &str = "work_item.resolve";
pub const WORK_ITEM_SYNC_CAPABILITY: &str = "work_item.sync";

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
    #[serde(default = "default_sync_interval_seconds")]
    pub sync_interval_seconds: u32,
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
            sync_interval_seconds: default_sync_interval_seconds(),
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

    pub fn with_sync_interval(mut self, seconds: u32) -> Result<Self, WorkItemError> {
        if !(1..=86_400).contains(&seconds) {
            return Err(WorkItemError::InvalidConfiguration);
        }
        self.sync_interval_seconds = seconds;
        Ok(self)
    }

    fn key(&self) -> WorkItemProviderKey {
        WorkItemProviderKey::new(&self.plugin_id, &self.host, &self.project)
    }
}

fn default_sync_interval_seconds() -> u32 {
    60
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct WorkItemIdentity {
    pub plugin_id: WorkItemProviderId,
    pub host: String,
    pub project: String,
    pub native_id: String,
}

impl WorkItemIdentity {
    pub fn validate(&self) -> Result<(), WorkItemError> {
        if self.plugin_id.as_str().trim().is_empty()
            || self.host.trim().is_empty()
            || self.project.trim().is_empty()
            || self.native_id.trim().is_empty()
            || self.host.contains('\0')
            || self.project.contains('\0')
            || self.native_id.contains('\0')
        {
            return Err(WorkItemError::InvalidSyncChange);
        }
        Ok(())
    }
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

    fn sync_capability(&self) -> Option<&WorkItemSyncCapability> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginWorkItemProvider {
    pub plugin_id: WorkItemProviderId,
    pub capabilities: WorkItemCapabilities,
    pub sync: Option<WorkItemSyncCapability>,
}

impl PluginWorkItemProvider {
    pub fn new(
        plugin_id: impl Into<String>,
        capabilities: WorkItemCapabilities,
    ) -> Result<Self, WorkItemError> {
        Ok(Self {
            plugin_id: WorkItemProviderId::new(plugin_id)?,
            capabilities,
            sync: None,
        })
    }

    pub fn with_sync_capability(
        mut self,
        sync: WorkItemSyncCapability,
    ) -> Result<Self, WorkItemError> {
        sync.validate()?;
        self.sync = Some(sync);
        Ok(self)
    }

    pub fn sync_capability(&self) -> Option<&WorkItemSyncCapability> {
        self.sync.as_ref()
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

    fn sync_capability(&self) -> Option<&WorkItemSyncCapability> {
        self.sync.as_ref()
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

/// Read the provider-owned status mapping. Core never infers a remote vocabulary.
pub async fn sync_capability_from_plugin(
    process: &crate::plugin::PluginProcess,
) -> Result<WorkItemSyncCapability, WorkItemError> {
    let response = process
        .call(WORK_ITEM_SYNC_CAPABILITY_METHOD, serde_json::json!({}))
        .await
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    let capability: WorkItemSyncCapability = serde_json::from_value(response)
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    capability.validate()?;
    Ok(capability)
}

/// Pull changes after the opaque cursor persisted by the host.
pub async fn pull_from_plugin(
    process: &crate::plugin::PluginProcess,
    identity: &WorkItemIdentity,
    cursor: Option<String>,
) -> Result<WorkItemPullResult, WorkItemError> {
    identity.validate()?;
    let response = process
        .call(
            WORK_ITEM_PULL_METHOD,
            serde_json::to_value(WorkItemPullRequest {
                identity: identity.clone(),
                cursor,
            })
            .map_err(|error| WorkItemError::Plugin(error.to_string()))?,
        )
        .await
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    let result: WorkItemPullResult = serde_json::from_value(response)
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    result.validate()?;
    Ok(result)
}

/// Ask the provider to apply its own mapping for a normalized local status.
pub async fn push_status_to_plugin(
    process: &crate::plugin::PluginProcess,
    request: &WorkItemPushStatusRequest,
) -> Result<(), WorkItemError> {
    request.identity.validate()?;
    process
        .call(
            WORK_ITEM_PUSH_STATUS_METHOD,
            serde_json::to_value(request)
                .map_err(|error| WorkItemError::Plugin(error.to_string()))?,
        )
        .await
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    Ok(())
}

/// Post an attributed local note; the provider owns the remote representation.
pub async fn push_note_to_plugin(
    process: &crate::plugin::PluginProcess,
    request: &WorkItemPushNoteRequest,
) -> Result<(), WorkItemError> {
    request.identity.validate()?;
    request.note.validate()?;
    process
        .call(
            WORK_ITEM_PUSH_NOTE_METHOD,
            serde_json::to_value(request)
                .map_err(|error| WorkItemError::Plugin(error.to_string()))?,
        )
        .await
        .map_err(|error| WorkItemError::Plugin(error.to_string()))?;
    Ok(())
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
    #[error("external work-item sync capability is invalid")]
    InvalidSyncCapability,
    #[error("external work-item sync change is invalid")]
    InvalidSyncChange,
    #[error("external status has no mapping for local column `{0:?}`")]
    UnmappedLocalStatus(TaskColumn),
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
    #[error("external work-item sync capability is required")]
    SyncCapabilityRequired,
    #[error("external work item is not imported")]
    ImportedWorkItemNotFound,
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkItemSyncState {
    pub pull_cursor: Option<String>,
    pub last_pushed_status: Option<String>,
    pub note_watermark: Option<String>,
    pub last_local_status_at: Option<String>,
    pub last_external_status_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub last_synced_at: Option<String>,
    pub unmapped_external_status: Option<String>,
    pub last_conflict_winner: Option<String>,
    pub last_conflict_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedWorkItem {
    pub local_task: BoardTask,
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
    #[serde(default)]
    pub sync_state: WorkItemSyncState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkItemSyncApplication {
    pub events: Vec<BoardEvent>,
    pub unmapped_statuses: Vec<String>,
    pub echo_suppressed_notes: Vec<String>,
    pub next_cursor: Option<String>,
    pub external_done_change_id: Option<String>,
    pub resolution_supported: bool,
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
        self.restore_imported_with_sync_state(
            task,
            snapshot,
            workflow,
            runs,
            evidence,
            WorkItemSyncState::default(),
        )
    }

    pub fn restore_imported_with_sync_state(
        &mut self,
        task: BoardTask,
        snapshot: WorkItemSnapshot,
        workflow: WorkflowSelection,
        runs: Vec<crate::services::task::TaskRunLink>,
        evidence: Vec<crate::services::task::TaskEvidenceLink>,
        sync_state: WorkItemSyncState,
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
                sync_state,
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
            sync_state: WorkItemSyncState::default(),
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

    pub fn sync_state(&self, identity: &WorkItemIdentity) -> Option<&WorkItemSyncState> {
        self.imported.get(identity).map(|item| &item.sync_state)
    }

    pub fn sync_state_mut(
        &mut self,
        identity: &WorkItemIdentity,
    ) -> Result<&mut WorkItemSyncState, WorkItemError> {
        self.imported
            .get_mut(identity)
            .map(|item| &mut item.sync_state)
            .ok_or(WorkItemError::TaskProjection)
    }

    pub fn local_status_push_request(
        &mut self,
        identity: &WorkItemIdentity,
        capability: &WorkItemSyncCapability,
        occurred_at: &str,
    ) -> Result<WorkItemPushStatusRequest, WorkItemError> {
        capability.validate()?;
        parse_sync_timestamp(occurred_at)?;
        let imported = self
            .imported
            .get_mut(identity)
            .ok_or(WorkItemError::ImportedWorkItemNotFound)?;
        capability
            .vocabulary
            .local_status(imported.local_task.column, imported.local_task.blocked)?;
        imported.sync_state.last_local_status_at = Some(occurred_at.into());
        imported.sync_state.last_sync_error = None;
        Ok(WorkItemPushStatusRequest {
            identity: identity.clone(),
            column: imported.local_task.column,
            blocked: imported.local_task.blocked,
        })
    }

    pub fn record_status_push(
        &mut self,
        identity: &WorkItemIdentity,
        external_status: &str,
    ) -> Result<(), WorkItemError> {
        if external_status.trim().is_empty() || external_status.contains('\0') {
            return Err(WorkItemError::InvalidSyncChange);
        }
        self.sync_state_mut(identity)?.last_pushed_status = Some(external_status.into());
        Ok(())
    }

    pub fn local_note_push_request(
        &self,
        identity: &WorkItemIdentity,
        id: impl Into<String>,
        body: impl Into<String>,
        author: impl Into<String>,
        occurred_at: impl Into<String>,
    ) -> Result<WorkItemPushNoteRequest, WorkItemError> {
        if !self.imported.contains_key(identity) {
            return Err(WorkItemError::ImportedWorkItemNotFound);
        }
        let id = id.into();
        let note = WorkItemNote {
            body: note_body_with_locus_marker(&body.into(), &id),
            id,
            author: author.into(),
            occurred_at: occurred_at.into(),
        };
        note.validate()?;
        Ok(WorkItemPushNoteRequest {
            identity: identity.clone(),
            note,
        })
    }

    pub fn append_local_note(
        &mut self,
        identity: &WorkItemIdentity,
        author: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<BoardComment, WorkItemError> {
        let author = author.into();
        let body = body.into();
        if author.trim().is_empty()
            || body.trim().is_empty()
            || author.contains('\0')
            || body.contains('\0')
        {
            return Err(WorkItemError::InvalidSyncChange);
        }
        let imported = self
            .imported
            .get(identity)
            .cloned()
            .ok_or(WorkItemError::ImportedWorkItemNotFound)?;
        let mut board = self.board.clone();
        let mut task = imported.local_task.clone();
        let comment = BoardComment {
            author,
            body,
            origin: BoardCommentOrigin::Local,
        };
        let event = BoardEvent::Commented {
            task_id: task.id,
            comment: comment.clone(),
            actor: BoardActor::Human,
        };
        board
            .apply(event)
            .map_err(|_| WorkItemError::TaskProjection)?;
        task = board
            .task(task.id)
            .cloned()
            .ok_or(WorkItemError::TaskProjection)?;
        let mut orchestrator = self.orchestrator.clone();
        orchestrator
            .update_task(task.clone())
            .map_err(|_| WorkItemError::TaskProjection)?;
        self.board = board;
        self.orchestrator = orchestrator;
        self.imported.insert(
            identity.clone(),
            ImportedWorkItem {
                local_task: task,
                snapshot: imported.snapshot,
                workflow: imported.workflow,
                sync_state: imported.sync_state,
            },
        );
        Ok(comment)
    }

    pub fn apply_pull(
        &mut self,
        identity: &WorkItemIdentity,
        capability: &WorkItemSyncCapability,
        result: WorkItemPullResult,
        synced_at: &str,
    ) -> Result<WorkItemSyncApplication, WorkItemError> {
        capability.validate()?;
        result.validate()?;
        parse_sync_timestamp(synced_at)?;
        let imported = self
            .imported
            .get(identity)
            .cloned()
            .ok_or(WorkItemError::ImportedWorkItemNotFound)?;
        let mut task = imported.local_task.clone();
        let mut snapshot = imported.snapshot.clone();
        let mut sync_state = imported.sync_state.clone();
        let mut board = self.board.clone();
        let mut orchestrator = self.orchestrator.clone();
        let mut application = WorkItemSyncApplication {
            next_cursor: result.next_cursor.clone(),
            ..Default::default()
        };

        for change in &result.changes {
            match change {
                WorkItemPullChange::Status {
                    id,
                    status,
                    occurred_at,
                    ..
                } => {
                    let occurred = parse_sync_timestamp(occurred_at)?;
                    let local = sync_state
                        .last_local_status_at
                        .as_deref()
                        .map(parse_sync_timestamp)
                        .transpose()?;
                    let external_wins = local.is_none_or(|local| occurred >= local);
                    snapshot.status = status.clone();
                    if sync_state
                        .last_external_status_at
                        .as_deref()
                        .is_none_or(|previous| occurred_at.as_str() > previous)
                    {
                        sync_state.last_external_status_at = Some(occurred_at.clone());
                    }
                    let Some(column) = capability.vocabulary.external_status(status) else {
                        sync_state.unmapped_external_status = Some(status.clone());
                        application.unmapped_statuses.push(status.clone());
                        continue;
                    };
                    sync_state.unmapped_external_status = None;
                    let conflict = local.is_some();
                    let winner = if external_wins { "external" } else { "local" };
                    if conflict {
                        sync_state.last_conflict_winner = Some(winner.into());
                        sync_state.last_conflict_reason =
                            Some("last-write-wins status conflict".into());
                    }
                    let target_column = if external_wins { column } else { task.column };
                    if external_wins && column == TaskColumn::Done {
                        application.external_done_change_id = Some(id.clone());
                    }
                    let should_record = conflict || task.column != target_column;
                    if should_record {
                        let evidence = BoardEvidenceLink {
                            run_id: None,
                            event_ids: Vec::new(),
                            artifact_ids: Vec::new(),
                            external: Some(BoardExternalEvidence {
                                provider: identity.plugin_id.as_str().into(),
                                native_id: identity.native_id.clone(),
                                change_id: id.clone(),
                                status: status.clone(),
                                occurred_at: occurred_at.clone(),
                                done: column == TaskColumn::Done,
                                winner: Some(winner.into()),
                                local_status_at: sync_state.last_local_status_at.clone(),
                                reason: Some(if conflict {
                                    "last-write-wins status conflict".into()
                                } else {
                                    "external status changed".into()
                                }),
                            }),
                        };
                        let event = task
                            .transition(
                                target_column,
                                BoardActor::Sync {
                                    provider: identity.plugin_id.as_str().into(),
                                },
                                vec![evidence],
                            )
                            .map_err(|_| WorkItemError::TaskProjection)?;
                        board
                            .apply(event.clone())
                            .map_err(|_| WorkItemError::TaskProjection)?;
                        application.events.push(event);
                        task = board
                            .task(task.id)
                            .cloned()
                            .ok_or(WorkItemError::TaskProjection)?;
                    }
                }
                WorkItemPullChange::Note {
                    id,
                    body,
                    author,
                    occurred_at,
                } => {
                    parse_sync_timestamp(occurred_at)?;
                    if locus_note_marker_id(body).is_some() {
                        application.echo_suppressed_notes.push(id.clone());
                    } else {
                        let event = BoardEvent::Commented {
                            task_id: task.id,
                            comment: BoardComment {
                                author: author.clone(),
                                body: body.clone(),
                                origin: BoardCommentOrigin::External {
                                    provider: identity.plugin_id.as_str().into(),
                                    note_id: id.clone(),
                                },
                            },
                            actor: BoardActor::Sync {
                                provider: identity.plugin_id.as_str().into(),
                            },
                        };
                        board
                            .apply(event.clone())
                            .map_err(|_| WorkItemError::TaskProjection)?;
                        application.events.push(event);
                        task = board
                            .task(task.id)
                            .cloned()
                            .ok_or(WorkItemError::TaskProjection)?;
                    }
                    if sync_state
                        .note_watermark
                        .as_deref()
                        .is_none_or(|watermark| occurred_at.as_str() > watermark)
                    {
                        sync_state.note_watermark = Some(occurred_at.clone());
                    }
                }
            }
        }

        sync_state.pull_cursor = result.next_cursor;
        sync_state.last_synced_at = Some(synced_at.into());
        sync_state.last_sync_error = None;
        if task.external_work_item.as_ref() != Some(&snapshot) {
            let event = BoardEvent::ExternalSnapshotUpdated {
                task_id: task.id,
                snapshot: snapshot.clone(),
            };
            board
                .apply(event.clone())
                .map_err(|_| WorkItemError::TaskProjection)?;
            application.events.push(event);
            task = board
                .task(task.id)
                .cloned()
                .ok_or(WorkItemError::TaskProjection)?;
        }
        orchestrator
            .update_task(task.clone())
            .map_err(|_| WorkItemError::TaskProjection)?;
        self.board = board;
        self.orchestrator = orchestrator;
        self.imported.insert(
            identity.clone(),
            ImportedWorkItem {
                local_task: task,
                snapshot,
                workflow: imported.workflow,
                sync_state,
            },
        );
        Ok(application)
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
        let sync = include_str!("../../../migrations/0026_external_work_item_sync.up.sql");
        assert!(sync.contains("pull_cursor"));
        assert!(sync.contains("sync_interval_seconds"));
        assert!(sync.contains("external_sync_changes"));
        assert!(sync.contains("task_comments"));
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
        assert_eq!(config("fixture.provider").sync_interval_seconds, 60);
        assert!(config("fixture.provider").with_sync_interval(0).is_err());
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

    fn sync_capability_fixture() -> WorkItemSyncCapability {
        WorkItemSyncCapability {
            vocabulary: WorkItemStatusVocabulary {
                external_to_local: BTreeMap::from([
                    ("open".into(), Some(TaskColumn::Ready)),
                    ("closed".into(), Some(TaskColumn::Done)),
                    ("triage".into(), None),
                ]),
                local_to_external: BTreeMap::from([
                    (TaskColumn::Ready, "open".into()),
                    (TaskColumn::InProgress, "open".into()),
                    (TaskColumn::Testing, "open".into()),
                    (TaskColumn::Reviewing, "open".into()),
                    (TaskColumn::PendingApproval, "open".into()),
                    (TaskColumn::Done, "closed".into()),
                ]),
                blocked_to_external: None,
            },
        }
    }

    #[test]
    fn sync_capability() {
        let provider = provider("fixture.provider", true)
            .with_sync_capability(sync_capability_fixture())
            .unwrap();
        let capability = provider.sync_capability().expect("sync capability");
        assert_eq!(
            capability.vocabulary.external_status("open"),
            Some(TaskColumn::Ready)
        );
        assert_eq!(
            capability
                .vocabulary
                .local_status(TaskColumn::Done, false)
                .unwrap(),
            "closed"
        );
        assert_eq!(
            capability.vocabulary.external_status("triage"),
            None,
            "unmapped statuses stay visible without a guessed column"
        );
    }

    #[test]
    fn sync_capability_rejects_empty_mapping_values() {
        let mut capability = sync_capability_fixture();
        capability
            .vocabulary
            .local_to_external
            .insert(TaskColumn::Done, String::new());
        assert_eq!(
            capability.validate(),
            Err(WorkItemError::InvalidSyncCapability)
        );
    }

    fn imported_for_sync() -> (WorkItemRegistry, WorkItemIdentity, TaskId) {
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
        let task_id = preview.workflow.task_id;
        registry.import_confirmed(preview).unwrap();
        (registry, identity, task_id)
    }

    #[test]
    fn sync_pull() {
        let (mut registry, identity, task_id) = imported_for_sync();
        let result = WorkItemPullResult {
            next_cursor: Some("2026-08-28T00:02:00Z".into()),
            changes: vec![
                WorkItemPullChange::Status {
                    id: "status-1".into(),
                    status: "closed".into(),
                    occurred_at: "2026-08-28T00:01:00Z".into(),
                    author: "octocat".into(),
                },
                WorkItemPullChange::Note {
                    id: "comment-1".into(),
                    body: "External context".into(),
                    occurred_at: "2026-08-28T00:01:30Z".into(),
                    author: "octocat".into(),
                },
            ],
        };
        let applied = registry
            .apply_pull(
                &identity,
                &sync_capability_fixture(),
                result,
                "2026-08-28T00:03:00Z",
            )
            .unwrap();
        assert_eq!(
            registry.board().task(task_id).unwrap().column,
            TaskColumn::Done
        );
        assert_eq!(registry.board().task(task_id).unwrap().comments.len(), 1);
        assert_eq!(
            applied.events.len(),
            3,
            "move, note, and snapshot fold events"
        );
        assert_eq!(applied.external_done_change_id.as_deref(), Some("status-1"));
        assert_eq!(
            registry
                .sync_state(&identity)
                .unwrap()
                .pull_cursor
                .as_deref(),
            Some("2026-08-28T00:02:00Z")
        );
        assert_eq!(
            registry
                .sync_state(&identity)
                .unwrap()
                .note_watermark
                .as_deref(),
            Some("2026-08-28T00:01:30Z")
        );
    }

    #[test]
    fn external_done_satisfied() {
        let (mut registry, identity, task_id) = imported_for_sync();
        let applied = registry
            .apply_pull(
                &identity,
                &sync_capability_fixture(),
                WorkItemPullResult {
                    next_cursor: Some("2026-08-28T00:20:00Z".into()),
                    changes: vec![WorkItemPullChange::Status {
                        id: "close-1".into(),
                        status: "closed".into(),
                        occurred_at: "2026-08-28T00:19:00Z".into(),
                        author: "octocat".into(),
                    }],
                },
                "2026-08-28T00:21:00Z",
            )
            .unwrap();
        assert_eq!(
            registry.board().task(task_id).unwrap().column,
            TaskColumn::Done
        );
        assert_eq!(applied.external_done_change_id.as_deref(), Some("close-1"));
        assert!(!applied.resolution_supported);
    }

    #[test]
    fn sync_push_status() {
        let (mut registry, identity, task_id) = imported_for_sync();
        let request = registry
            .local_status_push_request(
                &identity,
                &sync_capability_fixture(),
                "2026-08-28T00:04:00Z",
            )
            .unwrap();
        assert_eq!(request.identity, identity);
        assert_eq!(request.column, TaskColumn::Ready);
        assert!(!request.blocked);
        assert_eq!(
            registry
                .sync_state(&identity)
                .unwrap()
                .last_local_status_at
                .as_deref(),
            Some("2026-08-28T00:04:00Z")
        );
        assert_eq!(registry.board().task(task_id).unwrap().id, task_id);
    }

    #[test]
    fn sync_push_note() {
        let (registry, identity, _) = imported_for_sync();
        let request = registry
            .local_note_push_request(
                &identity,
                "note-42",
                "Local update",
                "human",
                "2026-08-28T00:05:00Z",
            )
            .unwrap();
        assert!(request.note.body.contains("<!-- locus-note:note-42 -->"));
    }

    #[test]
    fn local_note_enters_the_task_stream() {
        let (mut registry, identity, task_id) = imported_for_sync();
        let comment = registry
            .append_local_note(&identity, "human", "Local context")
            .unwrap();
        assert!(matches!(comment.origin, BoardCommentOrigin::Local));
        assert_eq!(registry.board().task(task_id).unwrap().comments.len(), 1);
    }

    #[test]
    fn echo_suppression() {
        let (mut registry, identity, task_id) = imported_for_sync();
        let result = WorkItemPullResult {
            next_cursor: Some("2026-08-28T00:06:00Z".into()),
            changes: vec![WorkItemPullChange::Note {
                id: "note-42".into(),
                body: note_body_with_locus_marker("Local update", "note-42"),
                occurred_at: "2026-08-28T00:05:30Z".into(),
                author: "human".into(),
            }],
        };
        let applied = registry
            .apply_pull(
                &identity,
                &sync_capability_fixture(),
                result,
                "2026-08-28T00:07:00Z",
            )
            .unwrap();
        assert_eq!(applied.echo_suppressed_notes, vec!["note-42"]);
        assert!(applied
            .events
            .iter()
            .all(|event| !matches!(event, BoardEvent::Commented { .. })));
        assert!(registry.board().task(task_id).unwrap().comments.is_empty());
    }

    #[test]
    fn status_conflict_lww() {
        let (mut registry, identity, task_id) = imported_for_sync();
        registry
            .local_status_push_request(
                &identity,
                &sync_capability_fixture(),
                "2026-08-28T00:10:00Z",
            )
            .unwrap();
        let applied = registry
            .apply_pull(
                &identity,
                &sync_capability_fixture(),
                WorkItemPullResult {
                    next_cursor: Some("2026-08-28T00:11:00Z".into()),
                    changes: vec![WorkItemPullChange::Status {
                        id: "status-old".into(),
                        status: "closed".into(),
                        occurred_at: "2026-08-28T00:09:00Z".into(),
                        author: "octocat".into(),
                    }],
                },
                "2026-08-28T00:12:00Z",
            )
            .unwrap();
        assert_eq!(
            registry.board().task(task_id).unwrap().column,
            TaskColumn::Ready
        );
        let BoardEvent::Moved { evidence, .. } = &applied.events[0] else {
            panic!("status conflict should be a task.moved event")
        };
        assert_eq!(
            evidence[0].external.as_ref().unwrap().winner.as_deref(),
            Some("local")
        );
        assert_eq!(
            evidence[0]
                .external
                .as_ref()
                .unwrap()
                .local_status_at
                .as_deref(),
            Some("2026-08-28T00:10:00Z")
        );
    }

    #[test]
    fn sync_note_marker_round_trip() {
        let body = note_body_with_locus_marker("Local note", "note-42");
        assert_eq!(locus_note_marker_id(&body), Some("note-42"));
        assert_eq!(locus_note_marker_id("external note"), None);
    }

    #[test]
    fn lookup_round_trip() {
        let snapshot = snapshot("fixture.provider");
        let lookup = WorkItemLookup::from(&snapshot.identity);
        assert_eq!(lookup.plugin_id, snapshot.identity.plugin_id);
        assert_eq!(lookup.native_id, "42");
    }
}
