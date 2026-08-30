use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use locus_core::{
    core::Core,
    harness::materialize::report::{reports_for_registry, MaterializationReport},
    ids::{ArtifactId, BotId, ProjectId, RoutineId, RunId, TaskId},
    lsp::{DescriptorPin, LspDiagnostic},
    plugin::{builtin_manifests, PluginKind, PluginProcess, WorkItemProviderDescriptor},
    repo::GitState,
    services::{
        agents::{seeded_definitions, AgentDefinition},
        artifact::{ArtifactComment, ArtifactContent, ArtifactKind, ArtifactRow, ArtifactStore},
        board::{BoardActor, BoardCommentOrigin, BoardEvidenceLink},
        bots::{
            Bot, BotContainerState, BotRoutine, RoutineAttribution, RoutineExecution,
            RoutineExecutionStatus,
        },
        lint::discover as discover_linters,
        manage::TaskColumn,
        task::TaskDetailSummary,
        telemetry::{now_timestamp, Event},
    },
    store::{work_items::PersistedExternalCompletionStatus, Store},
    work_item::{
        pull_from_plugin, push_note_to_plugin, push_status_to_plugin, snapshot_from_plugin,
        sync_capability_from_plugin, CompletionDelivery, ExternalWorkItemProvider,
        ImportedWorkItem, PluginWorkItemProvider, WorkItemError, WorkItemIdentity, WorkItemLookup,
        WorkItemPreview, WorkItemProviderConfig, WorkItemProviderId, WorkItemRegistry,
        WorkItemSnapshot, WorkItemSyncState,
    },
};
use serde::{Deserialize, Serialize};
use tauri::{
    ipc::Channel,
    menu::{Menu, MenuItem},
    Manager, State, WebviewUrl, WebviewWindowBuilder,
};

const MODEL_TIERS: [&str; 4] = ["low", "medium", "high", "xhigh"];
const HARNESS_REGISTRY: &str = "../../../harnesses";
const COMMAND_PALETTE_ACCELERATOR: &str = "CmdOrCtrl+K";
const GLOBAL_SEARCH_ACCELERATOR: &str = "CmdOrCtrl+P";

#[derive(Default)]
struct LspDiagnosticsSubscriptions {
    next_id: AtomicU64,
    active: Mutex<BTreeMap<u64, Arc<AtomicBool>>>,
}

impl LspDiagnosticsSubscriptions {
    fn start(&self) -> Result<(u64, Arc<AtomicBool>), IpcError> {
        let id = self
            .next_id
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let stop = Arc::new(AtomicBool::new(false));
        self.active
            .lock()
            .map_err(|_| IpcError::internal("LSP diagnostics subscription lock is poisoned"))?
            .insert(id, stop.clone());
        Ok((id, stop))
    }

    fn stop(&self, id: u64) -> Result<(), IpcError> {
        if let Some(stop) = self
            .active
            .lock()
            .map_err(|_| IpcError::internal("LSP diagnostics subscription lock is poisoned"))?
            .remove(&id)
        {
            stop.store(true, Ordering::Release);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum IpcErrorKind {
    InvalidArgument,
    NotFound,
    Internal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IpcError {
    kind: IpcErrorKind,
    message: String,
}

impl IpcError {
    fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            kind: IpcErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: IpcErrorKind::NotFound,
            message: message.into(),
        }
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        Self {
            kind: IpcErrorKind::Internal,
            message: error.to_string(),
        }
    }
}

fn webviews_per_window() -> usize {
    1
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemProviderRequest {
    plugin_id: String,
    host: String,
    project: String,
    #[serde(default)]
    sync_interval_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemProviderResponse {
    plugin_id: String,
    host: String,
    project: String,
    comments: bool,
    resolution_supported: bool,
    sync_supported: bool,
    sync_interval_seconds: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportedExternalWorkItemComment {
    author: String,
    body: String,
    origin: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportedExternalWorkItemTask {
    id: TaskId,
    project_id: ProjectId,
    repo_id: String,
    title: String,
    column: String,
    status: &'static str,
    verify_command: String,
    assignee: Option<String>,
    gate: String,
    stuck_iterations: Option<u32>,
    max_iterations: u32,
    tools: String,
    tokens: Option<String>,
    workflow_id: String,
    root_session_id: Option<String>,
    child_run_ids: Vec<String>,
    evidence_ids: Vec<String>,
    comments: Vec<ImportedExternalWorkItemComment>,
    external_link: String,
    external_host: String,
    completion_status: String,
    completion_attempts: u32,
    resolution_supported: bool,
    sync_supported: bool,
    sync_state: ExternalWorkItemSyncStateResponse,
}

fn desktop_task_column(column: TaskColumn) -> String {
    match column {
        TaskColumn::PendingApproval => "waiting_for_approval".into(),
        other => other.as_str().into(),
    }
}

fn imported_external_work_item_task(
    imported: &ImportedWorkItem,
    completion: Option<&PersistedExternalCompletionStatus>,
    detail: Option<&TaskDetailSummary>,
) -> Result<ImportedExternalWorkItemTask, IpcError> {
    let workflow_id = imported
        .workflow
        .workflow_def_id
        .ok_or_else(|| IpcError::internal("imported task workflow definition disappeared"))?;
    let provider = admitted_work_item_provider(imported.snapshot.identity.plugin_id.as_str())?;
    let task = &imported.local_task;
    Ok(ImportedExternalWorkItemTask {
        id: task.id,
        project_id: task.project_id,
        repo_id: task.repo_id.map(|id| id.to_string()).unwrap_or_default(),
        title: task.summary.clone(),
        column: desktop_task_column(task.column),
        status: if task.blocked { "blocked" } else { "ok" },
        verify_command: task.verify_command.clone().unwrap_or_default(),
        assignee: task.assigned_agent.map(|id| id.to_string()),
        gate: "workflow".into(),
        stuck_iterations: None,
        max_iterations: 3,
        tools: "read-only tools".into(),
        tokens: None,
        workflow_id: workflow_id.to_string(),
        root_session_id: detail
            .and_then(|detail| detail.root_session_id)
            .or(task.session_id)
            .map(|id| id.to_string()),
        child_run_ids: detail
            .map(|detail| {
                detail
                    .runs
                    .iter()
                    .map(|run| run.run_id.to_string())
                    .collect()
            })
            .unwrap_or_default(),
        evidence_ids: detail
            .map(|detail| {
                detail
                    .evidence
                    .iter()
                    .flat_map(|evidence| evidence.artifact_ids.iter())
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_else(|| {
                task.evidence
                    .iter()
                    .flat_map(|evidence| evidence.artifact_ids.iter())
                    .map(ToString::to_string)
                    .collect()
            }),
        comments: task
            .comments
            .iter()
            .map(|comment| ImportedExternalWorkItemComment {
                author: comment.author.clone(),
                body: comment.body.clone(),
                origin: if matches!(comment.origin, BoardCommentOrigin::External { .. }) {
                    "external"
                } else {
                    "local"
                },
            })
            .collect(),
        external_link: imported.snapshot.url.clone(),
        external_host: imported.snapshot.identity.host.clone(),
        completion_status: completion
            .map(|status| status.status.clone())
            .unwrap_or_else(|| "pending".into()),
        completion_attempts: completion.map_or(0, |status| status.attempts),
        resolution_supported: completion
            .map_or(provider.resolve, |status| status.resolution_supported),
        sync_supported: provider.sync,
        sync_state: sync_state_response(imported.sync_state.clone()),
    })
}

fn admitted_work_item_manifest(
    plugin_id: &str,
) -> Result<locus_core::plugin::PluginManifest, IpcError> {
    builtin_manifests()
        .into_iter()
        .find(|manifest| manifest.kind == PluginKind::Provider && manifest.id == plugin_id)
        .ok_or_else(|| {
            IpcError::not_found(format!("work-item plugin `{plugin_id}` is not admitted"))
        })
}

fn admitted_work_item_provider(plugin_id: &str) -> Result<WorkItemProviderDescriptor, IpcError> {
    let manifest = admitted_work_item_manifest(plugin_id)?;
    WorkItemProviderDescriptor::from_manifest(&manifest).map_err(IpcError::internal)
}

fn admitted_work_item_executable(plugin_id: &str) -> Result<PathBuf, IpcError> {
    let manifest = admitted_work_item_manifest(plugin_id)?;
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(manifest.executable))
}

fn negotiate_work_item_provider(
    catalog: &WorkItemProviderDescriptor,
    runtime: WorkItemProviderDescriptor,
    runtime_schema_versions: &BTreeMap<String, String>,
) -> Result<PluginWorkItemProvider, IpcError> {
    let catalog_manifest = &catalog.manifest;
    let runtime_manifest = &runtime.manifest;
    let capabilities_match = runtime_manifest.capabilities.len()
        == catalog_manifest.capabilities.len()
        && runtime_manifest
            .capabilities
            .iter()
            .all(|capability| catalog_manifest.capabilities.contains(capability));
    let permissions_match = runtime_manifest.permissions.len()
        == catalog_manifest.permissions.len()
        && runtime_manifest
            .permissions
            .iter()
            .all(|permission| catalog_manifest.permissions.contains(permission));
    let catalog_schema_versions = catalog.plugin_descriptor().schema_versions;
    if runtime_manifest.protocol != catalog_manifest.protocol
        || runtime_manifest.kind != catalog_manifest.kind
        || runtime_manifest.id != catalog_manifest.id
        || runtime_manifest.version != catalog_manifest.version
        || runtime_schema_versions != &catalog_schema_versions
        || !capabilities_match
        || !permissions_match
    {
        return Err(IpcError::internal(
            "work-item plugin runtime does not match its admitted manifest",
        ));
    }
    runtime.work_item_provider().map_err(IpcError::internal)
}

async fn negotiated_work_item_provider(
    process: &PluginProcess,
    catalog: &WorkItemProviderDescriptor,
) -> Result<PluginWorkItemProvider, IpcError> {
    let handshake = process
        .handshake(
            &[
                locus_core::work_item::WORK_ITEM_SNAPSHOT_CAPABILITY.into(),
                locus_core::work_item::WORK_ITEM_COMMENT_CAPABILITY.into(),
            ],
            &[
                locus_core::work_item::WORK_ITEM_SNAPSHOT_CAPABILITY,
                locus_core::work_item::WORK_ITEM_COMMENT_CAPABILITY,
                locus_core::work_item::WORK_ITEM_RESOLVE_CAPABILITY,
                locus_core::work_item::WORK_ITEM_SYNC_CAPABILITY,
            ],
        )
        .await
        .map_err(IpcError::internal)?;
    let runtime_descriptor = handshake.descriptor;
    let runtime = WorkItemProviderDescriptor::from_plugin_descriptor(&runtime_descriptor)
        .map_err(IpcError::internal)?;
    let provider =
        negotiate_work_item_provider(catalog, runtime, &runtime_descriptor.schema_versions)?;
    if !catalog.sync {
        return Ok(provider);
    }
    let sync = sync_capability_from_plugin(process)
        .await
        .map_err(work_item_ipc_error)?;
    provider
        .with_sync_capability(sync)
        .map_err(work_item_ipc_error)
}

async fn fetch_external_work_item_snapshot(
    lookup: WorkItemLookup,
) -> Result<(PluginWorkItemProvider, WorkItemSnapshot), IpcError> {
    let catalog = admitted_work_item_provider(lookup.plugin_id.as_str())?;
    let executable = admitted_work_item_executable(lookup.plugin_id.as_str())?;
    let process = PluginProcess::spawn(executable, Duration::from_secs(30))
        .await
        .map_err(IpcError::internal)?;
    let result = async {
        let provider = negotiated_work_item_provider(&process, &catalog).await?;
        let snapshot = snapshot_from_plugin(&process, &lookup)
            .await
            .map_err(work_item_ipc_error)?;
        provider
            .normalize(snapshot.clone())
            .map_err(work_item_ipc_error)?;
        Ok((provider, snapshot))
    }
    .await;
    let _ = process.shutdown().await;
    result
}

async fn spawn_negotiated_work_item_provider(
    plugin_id: &str,
) -> Result<(PluginProcess, PluginWorkItemProvider), IpcError> {
    let catalog = admitted_work_item_provider(plugin_id)?;
    if !catalog.sync {
        return Err(work_item_ipc_error(WorkItemError::SyncCapabilityRequired));
    }
    let executable = admitted_work_item_executable(plugin_id)?;
    let process = PluginProcess::spawn(executable, Duration::from_secs(30))
        .await
        .map_err(IpcError::internal)?;
    match negotiated_work_item_provider(&process, &catalog).await {
        Ok(provider) => Ok((process, provider)),
        Err(error) => {
            let _ = process.shutdown().await;
            Err(error)
        }
    }
}

async fn validate_import_preview(preview: &WorkItemPreview) -> Result<(), IpcError> {
    let lookup = WorkItemLookup::from(&preview.snapshot.identity);
    let (_, snapshot) = fetch_external_work_item_snapshot(lookup).await?;
    if snapshot != preview.snapshot {
        return Err(IpcError::invalid_argument(
            "external work-item preview is stale; load it again",
        ));
    }
    Ok(())
}

fn work_item_ipc_error(error: WorkItemError) -> IpcError {
    match error {
        WorkItemError::Persistence(_) | WorkItemError::Plugin(_) => IpcError::internal(error),
        _ => IpcError::invalid_argument(error.to_string()),
    }
}

fn validate_work_item_provider_config(config: &WorkItemProviderConfig) -> Result<(), IpcError> {
    if config.plugin_id.as_str() == "github" && config.host != "github.com" {
        return Err(IpcError::invalid_argument(
            "the GitHub work-item plugin only permits github.com",
        ));
    }
    Ok(())
}

fn external_work_item_provider_response(
    config: WorkItemProviderConfig,
    descriptor: WorkItemProviderDescriptor,
) -> ExternalWorkItemProviderResponse {
    ExternalWorkItemProviderResponse {
        plugin_id: config.plugin_id.as_str().into(),
        host: config.host,
        project: config.project,
        comments: descriptor.comments,
        resolution_supported: descriptor.resolve,
        sync_supported: descriptor.sync,
        sync_interval_seconds: config.sync_interval_seconds,
    }
}

async fn connected_store(core: &Core) -> Result<&Store, IpcError> {
    if let Some(store) = core.store() {
        return Ok(store);
    }
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| IpcError::internal("DATABASE_URL is not configured"))?;
    core.connect(&database_url)
        .await
        .map_err(IpcError::internal)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DispatchStopAllResponse {
    snapshot_id: String,
    stopped_runs: usize,
}

#[tauri::command]
async fn dispatch_stop_all(
    core: State<'_, Arc<Core>>,
    write_handoffs: Option<bool>,
) -> Result<DispatchStopAllResponse, IpcError> {
    let store = connected_store(&core).await?;
    let snapshot = store
        .stop_all_with_handoffs(write_handoffs.unwrap_or(true))
        .await
        .map_err(IpcError::internal)?;
    Ok(DispatchStopAllResponse {
        snapshot_id: snapshot.id.to_string(),
        stopped_runs: snapshot.run_ids.len(),
    })
}

async fn resolve_project_id(store: &Store, identifier: &str) -> Result<ProjectId, IpcError> {
    store
        .resolve_project_id(identifier)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::invalid_argument("active project was not found"))
}

#[tauri::command]
async fn external_work_item_providers(
    core: State<'_, Arc<Core>>,
) -> Result<Vec<ExternalWorkItemProviderResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let configs = store
        .load_external_work_item_providers()
        .await
        .map_err(IpcError::internal)?;
    configs
        .into_iter()
        .map(|config| {
            validate_work_item_provider_config(&config)?;
            let descriptor = admitted_work_item_provider(config.plugin_id.as_str())?;
            Ok(external_work_item_provider_response(config, descriptor))
        })
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemWorkflowResponse {
    id: String,
    name: String,
    version: i32,
}

#[tauri::command]
async fn external_work_item_workflows(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<ExternalWorkItemWorkflowResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    store
        .workflow_definition_summaries(project_id)
        .await
        .map(|definitions| {
            definitions
                .into_iter()
                .map(|(id, name, version)| ExternalWorkItemWorkflowResponse {
                    id: id.to_string(),
                    name,
                    version,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn external_work_item_tasks(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<ImportedExternalWorkItemTask>, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let project_id = match project_id {
        Some(value) => Some(resolve_project_id(store, &value).await?),
        None => None,
    };
    let imported = {
        let registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        registry
            .imported_tasks()
            .filter(|imported| project_id.is_none_or(|id| imported.local_task.project_id == id))
            .map(|imported| {
                let detail = registry
                    .orchestrator()
                    .detail(imported.workflow.task_id)
                    .map_err(IpcError::internal)?;
                Ok((imported.clone(), detail))
            })
            .collect::<Result<Vec<_>, IpcError>>()?
    };
    let mut responses = Vec::with_capacity(imported.len());
    for (imported, detail) in imported {
        let completion = store
            .external_completion_status(imported.local_task.id)
            .await
            .map_err(IpcError::internal)?;
        responses.push(imported_external_work_item_task(
            &imported,
            completion.as_ref(),
            Some(&detail),
        )?);
    }
    Ok(responses)
}

#[tauri::command]
async fn register_external_work_item_provider(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemProviderRequest,
) -> Result<ExternalWorkItemProviderResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let config = WorkItemProviderConfig::new(request.plugin_id, request.host, request.project)
        .map_err(|error| IpcError::invalid_argument(error.to_string()))?
        .with_sync_interval(request.sync_interval_seconds.unwrap_or(60))
        .map_err(|error| IpcError::invalid_argument(error.to_string()))?;
    validate_work_item_provider_config(&config)?;
    let descriptor = admitted_work_item_provider(config.plugin_id.as_str())?;
    let store = connected_store(&core).await?;
    store
        .save_external_work_item_provider(&config)
        .await
        .map_err(IpcError::internal)?;
    core.work_items()
        .lock()
        .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?
        .configure(config.clone());
    Ok(external_work_item_provider_response(config, descriptor))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemPreviewRequest {
    plugin_id: String,
    host: String,
    project: String,
    native_id: String,
    project_id: String,
    workflow_def_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemImportRequest {
    preview: WorkItemPreview,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
enum ExternalWorkItemImportResponse {
    Imported {
        task: Box<ImportedExternalWorkItemTask>,
    },
    Existing {
        #[serde(rename = "taskId")]
        task_id: TaskId,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemCompletionRequest {
    task_id: String,
    #[serde(default)]
    evidence: Vec<ArtifactId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemCompletionResponse {
    task_id: TaskId,
    status: String,
    attempts: u32,
    commented: bool,
    resolved: Option<bool>,
    resolution_supported: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemSyncStateResponse {
    pull_cursor: Option<String>,
    last_pushed_status: Option<String>,
    note_watermark: Option<String>,
    last_local_status_at: Option<String>,
    last_external_status_at: Option<String>,
    last_sync_error: Option<String>,
    last_synced_at: Option<String>,
    unmapped_external_status: Option<String>,
    last_conflict_winner: Option<String>,
    last_conflict_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemSyncResponse {
    task_id: TaskId,
    applied_events: usize,
    unmapped_statuses: Vec<String>,
    echo_suppressed_notes: Vec<String>,
    next_cursor: Option<String>,
    state: ExternalWorkItemSyncStateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemNoteRequest {
    task_id: String,
    id: String,
    body: String,
    author: String,
}

fn sync_state_response(state: WorkItemSyncState) -> ExternalWorkItemSyncStateResponse {
    ExternalWorkItemSyncStateResponse {
        pull_cursor: state.pull_cursor,
        last_pushed_status: state.last_pushed_status,
        note_watermark: state.note_watermark,
        last_local_status_at: state.last_local_status_at,
        last_external_status_at: state.last_external_status_at,
        last_sync_error: state.last_sync_error,
        last_synced_at: state.last_synced_at,
        unmapped_external_status: state.unmapped_external_status,
        last_conflict_winner: state.last_conflict_winner,
        last_conflict_reason: state.last_conflict_reason,
    }
}

fn restore_work_item_registry(core: &Core, registry: WorkItemRegistry) -> Result<(), IpcError> {
    *core
        .work_items()
        .lock()
        .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))? =
        registry;
    Ok(())
}

#[tauri::command]
async fn preview_external_work_item(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemPreviewRequest,
) -> Result<WorkItemPreview, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &request.project_id).await?;
    let workflow_def_id = request
        .workflow_def_id
        .map(|value| {
            value.parse().map_err(|error| {
                IpcError::invalid_argument(format!("invalid workflow definition id: {error}"))
            })
        })
        .transpose()?;
    if let Some(workflow_def_id) = workflow_def_id {
        if !store
            .workflow_definition_belongs_to_project(workflow_def_id, project_id)
            .await
            .map_err(IpcError::internal)?
        {
            return Err(IpcError::invalid_argument(
                "workflow definition does not belong to the active project",
            ));
        }
    }
    let plugin_id = WorkItemProviderId::new(request.plugin_id).map_err(work_item_ipc_error)?;
    let configured_identity = WorkItemIdentity {
        plugin_id: plugin_id.clone(),
        host: request.host.clone(),
        project: request.project.clone(),
        native_id: request.native_id.clone(),
    };
    {
        let registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        registry
            .select_for(&configured_identity)
            .map_err(work_item_ipc_error)?;
    }
    let lookup = WorkItemLookup::from(&configured_identity);
    let (_, snapshot) = fetch_external_work_item_snapshot(lookup).await?;
    let registry = core
        .work_items()
        .lock()
        .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
    let preview = registry
        .preview(snapshot, project_id, workflow_def_id)
        .map_err(work_item_ipc_error)?;
    drop(registry);
    core.pending_work_item_previews()
        .lock()
        .map_err(|_| IpcError::internal("pending work-item preview lock is poisoned"))?
        .insert(
            preview.workflow.task_id,
            (project_id, preview.snapshot.clone()),
        );
    Ok(preview)
}

#[tauri::command]
async fn import_external_work_item(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemImportRequest,
) -> Result<ExternalWorkItemImportResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let preview_id = request.preview.workflow.task_id;
    let identity = request.preview.snapshot.identity.clone();
    let pending = core
        .pending_work_item_previews()
        .lock()
        .map_err(|_| IpcError::internal("pending work-item preview lock is poisoned"))?
        .remove(&preview_id)
        .ok_or_else(|| {
            IpcError::invalid_argument(
                "external work-item preview is missing or has already been confirmed",
            )
        })?;
    if pending.0 != request.preview.workflow.project_id || pending.1 != request.preview.snapshot {
        return Err(IpcError::invalid_argument(
            "external work-item preview does not match the confirmed project or snapshot",
        ));
    }
    validate_import_preview(&request.preview).await?;
    if let Some(task_id) = store
        .external_work_item_task(&identity)
        .await
        .map_err(IpcError::internal)?
    {
        return Ok(ExternalWorkItemImportResponse::Existing { task_id });
    }
    let (before, imported) = {
        let mut registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        let before = registry.clone();
        let imported = registry
            .import_confirmed(request.preview)
            .map_err(work_item_ipc_error)?;
        (before, imported)
    };
    match store
        .persist_imported_task(&imported.local_task, &imported.snapshot, &imported.workflow)
        .await
    {
        Ok(true) => Ok(ExternalWorkItemImportResponse::Imported {
            task: Box::new(imported_external_work_item_task(&imported, None, None)?),
        }),
        Ok(false) => {
            restore_work_item_registry(&core, before)?;
            let task_id = store
                .external_work_item_task(&identity)
                .await
                .map_err(IpcError::internal)?
                .ok_or_else(|| IpcError::internal("duplicate external task disappeared"))?;
            Ok(ExternalWorkItemImportResponse::Existing { task_id })
        }
        Err(error) => {
            restore_work_item_registry(&core, before)?;
            Err(IpcError::internal(error))
        }
    }
}

fn completion_response(
    task_id: TaskId,
    delivery: &CompletionDelivery,
    resolution_supported: bool,
    error: Option<String>,
) -> ExternalWorkItemCompletionResponse {
    let status = if error.is_some() {
        "failed"
    } else if delivery.resolved == Some(true) {
        "resolved"
    } else if delivery.commented {
        "commented"
    } else {
        "pending"
    };
    ExternalWorkItemCompletionResponse {
        task_id,
        status: status.into(),
        attempts: delivery.attempts,
        commented: delivery.commented,
        resolved: delivery.resolved,
        resolution_supported,
        error,
    }
}

fn completion_response_from_status(
    task_id: TaskId,
    status: PersistedExternalCompletionStatus,
) -> ExternalWorkItemCompletionResponse {
    ExternalWorkItemCompletionResponse {
        task_id,
        status: status.status,
        attempts: status.attempts,
        commented: status.commented,
        resolved: status.resolved,
        resolution_supported: status.resolution_supported,
        error: status.last_error,
    }
}

fn completion_is_satisfied(status: &PersistedExternalCompletionStatus) -> bool {
    status.last_error.is_none() && status.commented && status.resolved != Some(false)
}

async fn deliver_external_work_item(
    core: &Core,
    store: &Store,
    task_id: TaskId,
    snapshot: WorkItemSnapshot,
    provider: PluginWorkItemProvider,
) -> Result<ExternalWorkItemCompletionResponse, IpcError> {
    if !store
        .external_task_is_done(task_id)
        .await
        .map_err(IpcError::internal)?
    {
        return Err(work_item_ipc_error(WorkItemError::NotDone));
    }
    let executable = admitted_work_item_executable(provider.plugin_id.as_str())?;
    let catalog_provider = admitted_work_item_provider(provider.plugin_id.as_str())?;
    let identity = snapshot.identity.clone();
    let mut delivery_error = None;
    let mut delivery_attempted = false;
    let mut resolution_supported = provider.capabilities.resolve;
    match PluginProcess::spawn(executable, Duration::from_secs(30)).await {
        Ok(process) => {
            let runtime_provider = negotiated_work_item_provider(&process, &catalog_provider)
                .await
                .map_err(|error| error.message);
            match runtime_provider {
                Ok(runtime_provider) => {
                    delivery_attempted = true;
                    resolution_supported = runtime_provider.capabilities.resolve;
                    if let Err(error) = core
                        .completion_outbox()
                        .lock()
                        .await
                        .deliver_via_plugin(task_id, &process, &runtime_provider, &identity)
                        .await
                    {
                        delivery_error = Some(error.to_string());
                    }
                }
                Err(error) => delivery_error = Some(error),
            }
            let _ = process.shutdown().await;
        }
        Err(error) => {
            delivery_error = Some(error.to_string());
        }
    }

    let delivery = {
        let mut outbox = core.completion_outbox().lock().await;
        if delivery_error.is_some() && !delivery_attempted {
            outbox
                .record_delivery_failure(task_id)
                .map_err(work_item_ipc_error)?;
        }
        outbox
            .delivery(task_id)
            .cloned()
            .ok_or_else(|| IpcError::internal("external completion delivery disappeared"))?
    };
    store
        .record_external_completion_attempt(
            delivery.event.id,
            i32::try_from(delivery.attempts)
                .map_err(|_| IpcError::internal("completion attempts exceed database range"))?,
            delivery.commented,
            delivery.resolved == Some(true),
            delivery_error.as_deref(),
        )
        .await
        .map_err(IpcError::internal)?;
    Ok(completion_response(
        task_id,
        &delivery,
        resolution_supported,
        delivery_error,
    ))
}

#[tauri::command]
async fn complete_external_work_item(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemCompletionRequest,
) -> Result<ExternalWorkItemCompletionResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let task_id = request
        .task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    if let Some(status) = store
        .external_completion_status(task_id)
        .await
        .map_err(IpcError::internal)?
    {
        if completion_is_satisfied(&status) {
            return Ok(completion_response_from_status(task_id, status));
        }
    }
    let (before_registry, from, task, snapshot, provider) =
        {
            let mut registry = core
                .work_items()
                .lock()
                .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
            let task =
                registry.board().task(task_id).cloned().ok_or_else(|| {
                    IpcError::not_found(format!("task `{task_id}` was not found"))
                })?;
            let snapshot = task.external_work_item.clone().ok_or_else(|| {
                IpcError::invalid_argument("task is not an imported GitHub issue")
            })?;
            registry
                .select_for(&snapshot.identity)
                .map_err(work_item_ipc_error)?;
            let provider = admitted_work_item_provider(snapshot.identity.plugin_id.as_str())?
                .work_item_provider()
                .map_err(IpcError::internal)?;
            let before_registry = registry.clone();
            let from = task.column;
            let task = if from == locus_core::services::manage::TaskColumn::Done {
                task
            } else {
                registry
                    .move_to_done(
                        task_id,
                        BoardActor::Human,
                        vec![BoardEvidenceLink {
                            run_id: None,
                            event_ids: Vec::new(),
                            artifact_ids: request.evidence.clone(),
                            external: None,
                        }],
                    )
                    .map_err(work_item_ipc_error)?
            };
            (before_registry, from, task, snapshot, provider)
        };
    let before_outbox = core.completion_outbox().lock().await.clone();
    let enqueue_result = {
        let mut outbox = core.completion_outbox().lock().await;
        outbox
            .enqueue_done_with_provider(&task, request.evidence, &provider)
            .map(|_| ())
            .map_err(work_item_ipc_error)
    };
    if let Err(error) = enqueue_result {
        restore_work_item_registry(&core, before_registry)?;
        return Err(error);
    }
    let delivery = core
        .completion_outbox()
        .lock()
        .await
        .delivery(task_id)
        .cloned()
        .ok_or_else(|| IpcError::internal("external completion was not enqueued"))?;
    if let Err(error) = store
        .persist_external_done_and_completion(&task, from, &delivery, &snapshot)
        .await
    {
        restore_work_item_registry(&core, before_registry)?;
        *core.completion_outbox().lock().await = before_outbox;
        return Err(IpcError::internal(error));
    }
    deliver_external_work_item(&core, store, task_id, snapshot, provider).await
}

#[tauri::command]
async fn retry_external_work_item_completion(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemCompletionRequest,
) -> Result<ExternalWorkItemCompletionResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let task_id = request
        .task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    if let Some(status) = store
        .external_completion_status(task_id)
        .await
        .map_err(IpcError::internal)?
    {
        if completion_is_satisfied(&status) {
            return Ok(completion_response_from_status(task_id, status));
        }
    }
    let (snapshot, provider) = {
        let registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        let task = registry
            .board()
            .task(task_id)
            .ok_or_else(|| IpcError::not_found(format!("task `{task_id}` was not found")))?;
        if task.column != TaskColumn::Done {
            return Err(work_item_ipc_error(WorkItemError::NotDone));
        }
        let snapshot = task
            .external_work_item
            .clone()
            .ok_or_else(|| IpcError::invalid_argument("task is not an imported GitHub issue"))?;
        registry
            .select_for(&snapshot.identity)
            .map_err(work_item_ipc_error)?;
        let provider = admitted_work_item_provider(snapshot.identity.plugin_id.as_str())?
            .work_item_provider()
            .map_err(IpcError::internal)?;
        (snapshot, provider)
    };
    deliver_external_work_item(&core, store, task_id, snapshot, provider).await
}

#[tauri::command]
async fn external_work_item_completion_status(
    core: State<'_, Arc<Core>>,
    task_id: String,
) -> Result<ExternalWorkItemCompletionResponse, IpcError> {
    let store = connected_store(&core).await?;
    let task_id = task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    let status = store
        .external_completion_status(task_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| {
            IpcError::not_found(format!("completion for task `{task_id}` was not found"))
        })?;
    Ok(ExternalWorkItemCompletionResponse {
        task_id,
        status: status.status,
        attempts: status.attempts,
        commented: status.commented,
        resolved: status.resolved,
        resolution_supported: status.resolution_supported,
        error: status.last_error,
    })
}

#[tauri::command]
async fn external_work_item_sync_state(
    core: State<'_, Arc<Core>>,
    task_id: String,
) -> Result<Option<ExternalWorkItemSyncStateResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let task_id = task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    store
        .external_sync_state(task_id)
        .await
        .map(|state| state.map(sync_state_response))
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn sync_external_work_item(
    core: State<'_, Arc<Core>>,
    task_id: String,
) -> Result<ExternalWorkItemSyncResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let task_id = task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    let (identity, before_registry, cursor) = {
        let registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        let task = registry
            .board()
            .task(task_id)
            .ok_or_else(|| IpcError::not_found(format!("task `{task_id}` was not found")))?;
        let identity = task
            .external_work_item
            .as_ref()
            .ok_or_else(|| IpcError::invalid_argument("task is not an imported work item"))?
            .identity
            .clone();
        let cursor = registry
            .sync_state(&identity)
            .and_then(|state| state.pull_cursor.clone());
        (identity, registry.clone(), cursor)
    };
    let (process, provider) =
        spawn_negotiated_work_item_provider(identity.plugin_id.as_str()).await?;
    let result: Result<ExternalWorkItemSyncResponse, IpcError> = async {
        let capability = provider
            .sync_capability()
            .cloned()
            .ok_or_else(|| work_item_ipc_error(WorkItemError::SyncCapabilityRequired))?;
        let pull = pull_from_plugin(&process, &identity, cursor)
            .await
            .map_err(work_item_ipc_error)?;
        let synced_at = now_timestamp();
        let (application, snapshot, state) = {
            let mut registry = core
                .work_items()
                .lock()
                .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
            let mut application = registry
                .apply_pull(&identity, &capability, pull, &synced_at)
                .map_err(work_item_ipc_error)?;
            application.resolution_supported = provider.capabilities.resolve;
            let imported = registry
                .imported(&identity)
                .ok_or_else(|| IpcError::internal("imported work item disappeared during sync"))?;
            (
                application,
                imported.snapshot.clone(),
                imported.sync_state.clone(),
            )
        };
        store
            .persist_external_sync(task_id, &snapshot, &application, &state)
            .await
            .map_err(IpcError::internal)?;
        Ok(ExternalWorkItemSyncResponse {
            task_id,
            applied_events: application.events.len(),
            unmapped_statuses: application.unmapped_statuses,
            echo_suppressed_notes: application.echo_suppressed_notes,
            next_cursor: application.next_cursor,
            state: sync_state_response(state),
        })
    }
    .await;
    let _ = process.shutdown().await;
    if let Err(error) = &result {
        restore_work_item_registry(&core, before_registry)?;
        let _ = store
            .record_external_sync_error(task_id, &error.message)
            .await;
    }
    result
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemStatusPushResponse {
    task_id: TaskId,
    external_status: String,
    state: ExternalWorkItemSyncStateResponse,
}

#[tauri::command]
async fn push_external_work_item_status(
    core: State<'_, Arc<Core>>,
    task_id: String,
) -> Result<ExternalWorkItemStatusPushResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let task_id = task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    let (identity, before_registry) = {
        let registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        let task = registry
            .board()
            .task(task_id)
            .ok_or_else(|| IpcError::not_found(format!("task `{task_id}` was not found")))?;
        let identity = task
            .external_work_item
            .as_ref()
            .ok_or_else(|| IpcError::invalid_argument("task is not an imported work item"))?
            .identity
            .clone();
        (identity, registry.clone())
    };
    let (process, provider) =
        spawn_negotiated_work_item_provider(identity.plugin_id.as_str()).await?;
    let result =
        async {
            let capability = provider
                .sync_capability()
                .cloned()
                .ok_or_else(|| work_item_ipc_error(WorkItemError::SyncCapabilityRequired))?;
            let occurred_at = now_timestamp();
            let request = {
                let mut registry = core.work_items().lock().map_err(|_| {
                    IpcError::internal("external work-item registry lock is poisoned")
                })?;
                registry
                    .local_status_push_request(&identity, &capability, &occurred_at)
                    .map_err(work_item_ipc_error)?
            };
            push_status_to_plugin(&process, &request)
                .await
                .map_err(work_item_ipc_error)?;
            let external_status = capability
                .vocabulary
                .local_status(request.column, request.blocked)
                .map_err(work_item_ipc_error)?
                .to_owned();
            let state = {
                let mut registry = core.work_items().lock().map_err(|_| {
                    IpcError::internal("external work-item registry lock is poisoned")
                })?;
                registry
                    .record_status_push(&identity, &external_status)
                    .map_err(work_item_ipc_error)?;
                registry
                    .sync_state(&identity)
                    .cloned()
                    .ok_or_else(|| IpcError::internal("external sync state disappeared"))?
            };
            if !store
                .save_external_sync_state(task_id, &state)
                .await
                .map_err(IpcError::internal)?
            {
                return Err(IpcError::internal(
                    "external work item disappeared during status push",
                ));
            }
            Ok(ExternalWorkItemStatusPushResponse {
                task_id,
                external_status,
                state: sync_state_response(state),
            })
        }
        .await;
    let _ = process.shutdown().await;
    if let Err(error) = &result {
        restore_work_item_registry(&core, before_registry)?;
        let _ = store
            .record_external_sync_error(task_id, &error.message)
            .await;
    }
    result
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalWorkItemNotePushResponse {
    task_id: TaskId,
    posted: bool,
}

#[tauri::command]
async fn push_external_work_item_note(
    core: State<'_, Arc<Core>>,
    request: ExternalWorkItemNoteRequest,
) -> Result<ExternalWorkItemNotePushResponse, IpcError> {
    let _operation_lock = core.work_item_operation_lock().lock().await;
    let store = connected_store(&core).await?;
    let task_id = request
        .task_id
        .parse::<TaskId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid task id: {error}")))?;
    let (identity, before_registry, note, comment) = {
        let mut registry = core
            .work_items()
            .lock()
            .map_err(|_| IpcError::internal("external work-item registry lock is poisoned"))?;
        let identity = registry
            .board()
            .task(task_id)
            .and_then(|task| task.external_work_item.as_ref())
            .map(|snapshot| snapshot.identity.clone())
            .ok_or_else(|| IpcError::invalid_argument("task is not an imported work item"))?;
        let before_registry = registry.clone();
        let note = registry
            .local_note_push_request(
                &identity,
                request.id.clone(),
                request.body.clone(),
                request.author.clone(),
                now_timestamp(),
            )
            .map_err(work_item_ipc_error)?;
        let comment = registry
            .append_local_note(&identity, request.author, request.body)
            .map_err(work_item_ipc_error)?;
        (identity, before_registry, note, comment)
    };
    if let Err(error) = store
        .persist_local_task_comment(task_id, &note.note.id, &comment)
        .await
    {
        restore_work_item_registry(&core, before_registry)?;
        return Err(IpcError::internal(error));
    }
    let (process, _provider) =
        spawn_negotiated_work_item_provider(identity.plugin_id.as_str()).await?;
    let result = push_note_to_plugin(&process, &note)
        .await
        .map_err(work_item_ipc_error);
    let _ = process.shutdown().await;
    result.map(|()| ExternalWorkItemNotePushResponse {
        task_id,
        posted: true,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTierGridRequest {
    pub project_id: String,
    pub harnesses: Vec<HarnessTierGridHarnessRequest>,
    pub tier_settings: Vec<ModelTierSetting>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTierGridHarnessRequest {
    pub name: String,
    pub models: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTierSetting {
    pub harness: String,
    pub tier: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTierGridResponse {
    pub project_id: String,
    pub harnesses: Vec<HarnessTierGridHarness>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefSummary {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefResponse {
    pub name: String,
    pub version: u32,
    pub frontmatter: serde_json::Value,
    pub body: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactResponse {
    id: String,
    run_id: String,
    kind: String,
    title: String,
    body: Option<String>,
    blob_path: Option<String>,
    media_type: String,
    sha256: String,
    derived_text: Option<String>,
    /// The in-memory fixture seam has no durable creation timestamp.
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactCommentResponse {
    id: String,
    artifact_id: String,
    parent_id: Option<String>,
    author: String,
    body: String,
    /// The in-memory fixture seam has no durable creation timestamp.
    created_at: Option<String>,
}

fn artifact_kind_name(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Plan => "plan",
        ArtifactKind::Diff => "diff",
        ArtifactKind::Diagram => "diagram",
        ArtifactKind::Image => "image",
        ArtifactKind::Recording => "recording",
        ArtifactKind::Walkthrough => "walkthrough",
        ArtifactKind::Finding => "finding",
        ArtifactKind::Payload => "payload",
    }
}

fn artifact_response(row: &ArtifactRow) -> ArtifactResponse {
    let (body, blob_path, media_type, sha256) = match &row.content {
        ArtifactContent::Text(body) => {
            (Some(body.clone()), None, "text/plain".into(), String::new())
        }
        ArtifactContent::Blob {
            path,
            media_type,
            sha256,
        } => (
            None,
            Some(path.display().to_string()),
            media_type.clone(),
            sha256.clone(),
        ),
    };
    let kind = artifact_kind_name(row.kind).to_owned();
    ArtifactResponse {
        id: row.id.to_string(),
        run_id: row.run_id.to_string(),
        title: format!("{kind} {}", row.id),
        kind,
        body,
        blob_path,
        media_type,
        sha256,
        derived_text: row.derived_cache.as_ref().map(ToString::to_string),
        created_at: None,
    }
}

fn artifact_comment_response(comment: &ArtifactComment) -> ArtifactCommentResponse {
    ArtifactCommentResponse {
        id: comment.id.to_string(),
        artifact_id: comment.artifact_id.to_string(),
        parent_id: comment.parent_id.map(|id| id.to_string()),
        author: "human".into(),
        body: comment.body.clone(),
        created_at: None,
    }
}

fn seeded_artifact_store() -> ArtifactStore {
    let project_id = ProjectId::generate();
    let run_id = RunId::generate();
    let mut store = ArtifactStore::default();
    let diff = ArtifactRow::text(
        project_id,
        run_id,
        ArtifactKind::Diff,
        "diff --git a/src/lib.rs b/src/lib.rs\n+real artifact data reaches Review",
    );
    let diff_id = diff.id;
    store.put(diff);
    store.put(ArtifactRow::text(
        project_id,
        run_id,
        ArtifactKind::Plan,
        "Wire the Review Artifacts screen to the core artifact store.",
    ));
    store
        .comment(
            diff_id,
            None,
            "Review this artifact from the live core store.",
        )
        .expect("seeded artifact exists");
    store
}

#[tauri::command]
fn artifacts_list(artifacts: State<'_, ArtifactStore>) -> Vec<ArtifactResponse> {
    artifacts
        .review_inbox()
        .into_iter()
        .map(artifact_response)
        .collect()
}

#[tauri::command]
fn artifact_comments(
    artifacts: State<'_, ArtifactStore>,
    artifact_id: String,
) -> Result<Vec<ArtifactCommentResponse>, IpcError> {
    let artifact_id: ArtifactId = artifact_id.parse().map_err(IpcError::internal)?;
    Ok(artifacts
        .comments(artifact_id)
        .into_iter()
        .map(artifact_comment_response)
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotResponse {
    id: String,
    project_id: String,
    name: String,
    agent_def_id: String,
    home_session_id: String,
    branch: String,
    container_id: Option<String>,
    container_state: BotContainerState,
    warm_until: Option<String>,
    last_activity_at: Option<String>,
    total_cost_micros: Option<u64>,
}

impl From<Bot> for BotResponse {
    fn from(bot: Bot) -> Self {
        Self {
            id: bot.id.to_string(),
            project_id: bot.project_id.to_string(),
            name: bot.name,
            agent_def_id: bot.agent_def_id.to_string(),
            home_session_id: bot.home_session_id.to_string(),
            branch: bot.branch,
            container_id: bot.container_id,
            container_state: bot.container_state,
            warm_until: bot.warm_until,
            last_activity_at: bot.last_activity_at,
            total_cost_micros: bot.total_cost_micros,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotRoutineResponse {
    id: String,
    bot_id: String,
    prompt: String,
    cron_expression: String,
    enabled: bool,
    skipped_count: u32,
    schedule_id: Option<String>,
}

impl From<BotRoutine> for BotRoutineResponse {
    fn from(routine: BotRoutine) -> Self {
        Self {
            id: routine.id.to_string(),
            bot_id: routine.bot_id.to_string(),
            prompt: routine.prompt,
            cron_expression: routine.cron_expression,
            enabled: routine.enabled,
            skipped_count: routine.skipped_count,
            schedule_id: routine.schedule_id.map(|id| id.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotRoutineExecutionResponse {
    id: String,
    bot_id: String,
    scheduled_for: i64,
    status: RoutineExecutionStatus,
    result: Option<locus_core::services::bots::RoutineResult>,
    attribution: RoutineAttribution,
    test_run: bool,
}

impl From<RoutineExecution> for BotRoutineExecutionResponse {
    fn from(execution: RoutineExecution) -> Self {
        Self {
            id: execution.id.to_string(),
            bot_id: execution.bot_id.to_string(),
            scheduled_for: execution.scheduled_for,
            status: execution.status,
            result: execution.result,
            attribution: execution.attribution,
            test_run: execution.test_run,
        }
    }
}

fn parse_bot_id(value: &str) -> Result<BotId, IpcError> {
    value
        .parse()
        .map_err(|error| IpcError::invalid_argument(format!("invalid bot id: {error}")))
}

fn parse_routine_id(value: &str) -> Result<RoutineId, IpcError> {
    value
        .parse()
        .map_err(|error| IpcError::invalid_argument(format!("invalid routine id: {error}")))
}

#[tauri::command]
async fn bots_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<BotResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    store
        .bots(project_id)
        .await
        .map(|bots| bots.into_iter().map(BotResponse::from).collect())
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_create(
    core: State<'_, Arc<Core>>,
    project_id: String,
    markdown: String,
) -> Result<BotResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    store
        .create_bot_from_markdown(project_id, &markdown)
        .await
        .map(BotResponse::from)
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routines(
    core: State<'_, Arc<Core>>,
    bot_id: String,
) -> Result<Vec<BotRoutineResponse>, IpcError> {
    let store = connected_store(&core).await?;
    store
        .bot_routines(parse_bot_id(&bot_id)?)
        .await
        .map(|routines| routines.into_iter().map(BotRoutineResponse::from).collect())
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_executions(
    core: State<'_, Arc<Core>>,
    bot_id: String,
) -> Result<Vec<BotRoutineExecutionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    store
        .bot_routine_executions(parse_bot_id(&bot_id)?)
        .await
        .map(|executions| {
            executions
                .into_iter()
                .map(BotRoutineExecutionResponse::from)
                .collect()
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_set_enabled(
    core: State<'_, Arc<Core>>,
    routine_id: String,
    enabled: bool,
) -> Result<(), IpcError> {
    let store = connected_store(&core).await?;
    store
        .set_bot_routine_enabled(parse_routine_id(&routine_id)?, enabled)
        .await
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_update(
    core: State<'_, Arc<Core>>,
    routine_id: String,
    prompt: String,
    cron_expression: String,
) -> Result<BotRoutineResponse, IpcError> {
    let store = connected_store(&core).await?;
    store
        .update_bot_routine(parse_routine_id(&routine_id)?, &prompt, &cron_expression)
        .await
        .map(BotRoutineResponse::from)
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_delete(
    core: State<'_, Arc<Core>>,
    routine_id: String,
) -> Result<(), IpcError> {
    let store = connected_store(&core).await?;
    store
        .delete_bot_routine(parse_routine_id(&routine_id)?)
        .await
        .map_err(IpcError::internal)
}

fn seeded_agent_definitions() -> Vec<(u32, AgentDefinition)> {
    // M1 has no editable Workshop form yet. The screen reads the same core-owned
    // seed definitions that later migrate into agents.agent_defs, never fixtures.
    seeded_definitions()
        .into_iter()
        .map(|definition| (1, definition))
        .collect()
}

#[tauri::command]
fn agent_defs_list() -> Vec<AgentDefSummary> {
    seeded_agent_definitions()
        .into_iter()
        .map(|(version, definition)| AgentDefSummary {
            name: definition.frontmatter.name,
            version,
        })
        .collect()
}

#[tauri::command]
fn agent_def(name: String) -> Result<AgentDefResponse, IpcError> {
    let (version, definition) = seeded_agent_definitions()
        .into_iter()
        .find(|(_, definition)| definition.frontmatter.name == name)
        .ok_or_else(|| IpcError::not_found(format!("agent definition `{name}` was not found")))?;
    let frontmatter = serde_json::to_value(&definition.frontmatter).map_err(IpcError::internal)?;
    Ok(AgentDefResponse {
        name: definition.frontmatter.name,
        version,
        frontmatter,
        body: definition.body,
        warnings: definition.warnings,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTierGridHarness {
    pub name: String,
    pub models: Option<Vec<String>>,
    pub tiers: Vec<ModelTierSetting>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspAttachRequest {
    pub project_root: String,
    pub pane_id: String,
    pub file_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspAttachResponse {
    pub project_root: String,
    pub pane_id: String,
    pub descriptor_id: String,
}

#[tauri::command]
fn lsp_attach(
    core: State<'_, Arc<Core>>,
    request: LspAttachRequest,
) -> Result<LspAttachResponse, IpcError> {
    let pane = core
        .lsp()
        .attach(&request.project_root, request.pane_id, &request.file_path)
        .map_err(IpcError::internal)?;
    let descriptor = core
        .lsp()
        .descriptor_for_project_path(
            &pane.project_root,
            &std::path::Path::new(&request.project_root).join(&request.file_path),
        )
        .map_err(IpcError::internal)?;
    Ok(LspAttachResponse {
        project_root: pane.project_root.display().to_string(),
        pane_id: pane.pane_id,
        descriptor_id: descriptor.id,
    })
}

#[tauri::command]
async fn lsp_enable_descriptor(
    core: State<'_, Arc<Core>>,
    project_root: String,
    pin: DescriptorPin,
    project_id: Option<String>,
) -> Result<String, IpcError> {
    let project_root = std::fs::canonicalize(&project_root).map_err(IpcError::internal)?;
    let descriptor = core
        .lsp()
        .catalog()
        .descriptor_for_pin(&pin)
        .map_err(IpcError::internal)?;
    if let Some(project_id) = project_id {
        let project_id = project_id
            .parse::<ProjectId>()
            .map_err(|error| IpcError::invalid_argument(format!("invalid project id: {error}")))?;
        let store = core
            .store()
            .ok_or_else(|| IpcError::internal("project settings store is not connected"))?;
        let mut pins = store
            .project_lsp_descriptors(project_id)
            .await
            .map_err(IpcError::internal)?;
        pins.insert(pin.id.clone(), pin.clone());
        store
            .set_project_lsp_descriptors(project_id, pins.into_values())
            .await
            .map_err(IpcError::internal)?;
    }
    core.lsp()
        .enable_project_descriptor(project_root, pin)
        .map(|_| descriptor.id)
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn lsp_disable_descriptor(
    core: State<'_, Arc<Core>>,
    project_root: String,
    descriptor_id: String,
    project_id: Option<String>,
) -> Result<(), IpcError> {
    let project_root = std::fs::canonicalize(&project_root).map_err(IpcError::internal)?;
    if let Some(project_id) = project_id {
        let project_id = project_id
            .parse::<ProjectId>()
            .map_err(|error| IpcError::invalid_argument(format!("invalid project id: {error}")))?;
        let store = core
            .store()
            .ok_or_else(|| IpcError::internal("project settings store is not connected"))?;
        let mut pins = store
            .project_lsp_descriptors(project_id)
            .await
            .map_err(IpcError::internal)?;
        pins.remove(&descriptor_id);
        store
            .set_project_lsp_descriptors(project_id, pins.into_values())
            .await
            .map_err(IpcError::internal)?;
    }
    core.lsp()
        .disable_project_descriptor(project_root, &descriptor_id)
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn lsp_load_project_descriptors(
    core: State<'_, Arc<Core>>,
    project_root: String,
    project_id: String,
) -> Result<Vec<String>, IpcError> {
    let project_id = project_id
        .parse::<ProjectId>()
        .map_err(|error| IpcError::invalid_argument(format!("invalid project id: {error}")))?;
    let store = core
        .store()
        .ok_or_else(|| IpcError::internal("project settings store is not connected"))?;
    let pins = store
        .project_lsp_descriptors(project_id)
        .await
        .map_err(IpcError::internal)?;
    let mut ids = Vec::with_capacity(pins.len());
    for pin in pins.into_values() {
        ids.push(
            core.lsp()
                .enable_project_descriptor(&project_root, pin)
                .map_err(IpcError::internal)?
                .id,
        );
    }
    Ok(ids)
}

#[tauri::command]
fn lsp_detach(
    core: State<'_, Arc<Core>>,
    project_root: String,
    pane_id: String,
) -> Result<(), IpcError> {
    core.lsp()
        .detach(project_root, pane_id)
        .map_err(IpcError::internal)
}

#[tauri::command]
fn lsp_request(
    core: State<'_, Arc<Core>>,
    project_root: String,
    method: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, IpcError> {
    core.lsp()
        .request(project_root, &method, params)
        .map_err(IpcError::internal)
}

#[tauri::command]
fn lsp_notify(
    core: State<'_, Arc<Core>>,
    project_root: String,
    method: String,
    params: serde_json::Value,
) -> Result<(), IpcError> {
    core.lsp()
        .notify(project_root, &method, params)
        .map_err(IpcError::internal)
}

#[tauri::command]
fn lsp_diagnostics_subscribe(
    core: State<'_, Arc<Core>>,
    subscriptions: State<'_, Arc<LspDiagnosticsSubscriptions>>,
    project_root: String,
    channel: Channel<LspDiagnostic>,
) -> Result<u64, IpcError> {
    let project_root = std::fs::canonicalize(project_root).map_err(IpcError::internal)?;
    let core = core.inner().clone();
    let mut diagnostics = core.lsp().subscribe_diagnostics();
    let (id, stop) = subscriptions.start()?;
    let subscriptions = subscriptions.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(50));
        loop {
            ticker.tick().await;
            if stop.load(Ordering::Acquire) || core.lsp().poll_notifications().is_err() {
                break;
            }
            while let Ok(diagnostic) = diagnostics.try_recv() {
                if diagnostic.project_root != project_root {
                    continue;
                }
                if channel.send(diagnostic).is_err() {
                    stop.store(true, Ordering::Release);
                    break;
                }
            }
        }
        let _ = subscriptions.stop(id);
    });
    Ok(id)
}

#[tauri::command]
fn lsp_diagnostics_unsubscribe(
    subscriptions: State<'_, Arc<LspDiagnosticsSubscriptions>>,
    subscription_id: u64,
) -> Result<(), IpcError> {
    subscriptions.stop(subscription_id)
}

#[tauri::command]
fn pty_subscribe(core: State<'_, Arc<Core>>, channel: Channel<Vec<u8>>) {
    let mut bytes = core.pty().subscribe();
    tauri::async_runtime::spawn(async move {
        while let Ok(bytes) = bytes.recv().await {
            if channel.send(bytes).is_err() {
                break;
            }
        }
    });
}

#[tauri::command]
fn telemetry_subscribe(core: State<'_, Arc<Core>>, channel: Channel<Event>) {
    let mut events = core.collector().subscribe();
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = events.recv().await {
            if channel.send(event).is_err() {
                break;
            }
        }
    });
}

/// Replays a run's durable telemetry events from `agents.events` in capture order.
/// The live subscription only carries events since connect; this is how Telemetry
/// and Analytics rebuild a run's history after a restart.
#[tauri::command]
async fn telemetry_events_replay(
    core: State<'_, Arc<Core>>,
    run_id: String,
) -> Result<Vec<Event>, IpcError> {
    let run_id: RunId = run_id
        .parse()
        .map_err(|_| IpcError::internal("run id must be a UUID"))?;
    let store = connected_store(&core).await?;
    store
        .events_for_run(run_id)
        .await
        .map_err(IpcError::internal)
}

#[tauri::command]
fn linter_count(root: String) -> Result<usize, IpcError> {
    discover_linters(root)
        .map(|linters| linters.len())
        .map_err(IpcError::internal)
}

#[tauri::command]
fn repo_git_state(path: String) -> Result<GitState, IpcError> {
    locus_core::repo::RepoManager::default()
        .git_state(path)
        .map_err(IpcError::internal)
}

#[tauri::command]
fn materialization_report(
    core: State<'_, Arc<Core>>,
) -> Result<Vec<MaterializationReport>, IpcError> {
    // The registry is parsed once at start, not re-read and re-parsed per invoke.
    Ok(reports_for_registry(core.registry()))
}

#[tauri::command]
fn harness_tier_grid(request: HarnessTierGridRequest) -> Result<HarnessTierGridResponse, IpcError> {
    if request.project_id.trim().is_empty() {
        return Err(IpcError::invalid_argument(
            "harness tier grid requires a project id",
        ));
    }
    for setting in &request.tier_settings {
        if !MODEL_TIERS.contains(&setting.tier.as_str()) {
            return Err(IpcError::invalid_argument(format!(
                "unknown model tier `{}`",
                setting.tier
            )));
        }
    }
    let harnesses = request
        .harnesses
        .into_iter()
        .map(|harness| HarnessTierGridHarness {
            tiers: MODEL_TIERS
                .iter()
                .map(|tier| ModelTierSetting {
                    harness: harness.name.clone(),
                    tier: (*tier).into(),
                    model: request
                        .tier_settings
                        .iter()
                        .find(|setting| setting.harness == harness.name && setting.tier == *tier)
                        .and_then(|setting| setting.model.clone()),
                })
                .collect(),
            name: harness.name,
            models: harness.models,
        })
        .collect();
    Ok(HarnessTierGridResponse {
        project_id: request.project_id,
        harnesses,
    })
}

#[tauri::command]
fn detach_pane(app: tauri::AppHandle, pane_id: String) -> Result<(), IpcError> {
    let label = format!("pane-{pane_id}");
    if app.get_webview_window(&label).is_none() {
        WebviewWindowBuilder::new(
            &app,
            label,
            WebviewUrl::App("index.html?detached=true".into()),
        )
        .title("Locus pane")
        .build()
        .map_err(IpcError::internal)?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The composition root: one graph, built once. PLAN.md §Process topology.
    let core = Core::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(HARNESS_REGISTRY))
        .expect("load the harness registry at start");

    tauri::Builder::default()
        .manage(core)
        .manage(Arc::new(LspDiagnosticsSubscriptions::default()))
        .manage(seeded_artifact_store())
        .setup(|app| {
            let command_palette = MenuItem::with_id(
                app,
                "command-palette",
                "Command Palette",
                true,
                Some(COMMAND_PALETTE_ACCELERATOR),
            )?;
            let global_search = MenuItem::with_id(
                app,
                "global-search",
                "Search Everything",
                true,
                Some(GLOBAL_SEARCH_ACCELERATOR),
            )?;
            app.set_menu(Menu::with_items(app, &[&command_palette, &global_search])?)?;
            debug_assert_eq!(webviews_per_window(), 1);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            agent_def,
            agent_defs_list,
            harness_tier_grid,
            pty_subscribe,
            telemetry_subscribe,
            telemetry_events_replay,
            lsp_attach,
            lsp_enable_descriptor,
            lsp_disable_descriptor,
            lsp_load_project_descriptors,
            lsp_detach,
            lsp_request,
            lsp_notify,
            lsp_diagnostics_subscribe,
            lsp_diagnostics_unsubscribe,
            detach_pane,
            linter_count,
            artifacts_list,
            artifact_comments,
            bots_list,
            bot_create,
            bot_routines,
            bot_routine_executions,
            bot_routine_set_enabled,
            bot_routine_update,
            bot_routine_delete,
            dispatch_stop_all,
            external_work_item_providers,
            external_work_item_workflows,
            external_work_item_tasks,
            register_external_work_item_provider,
            preview_external_work_item,
            import_external_work_item,
            complete_external_work_item,
            retry_external_work_item_completion,
            external_work_item_completion_status,
            external_work_item_sync_state,
            sync_external_work_item,
            push_external_work_item_status,
            push_external_work_item_note,
            materialization_report,
            repo_git_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    mod menu {
        use super::*;

        #[test]
        fn no_default_key_equivalents() {
            // `Menu::with_items` receives only this app-owned item, rather than an OS default menu.
            assert_eq!(COMMAND_PALETTE_ACCELERATOR, "CmdOrCtrl+K");
        }
    }

    mod window {
        use super::*;

        #[test]
        fn one_webview_each() {
            assert_eq!(webviews_per_window(), 1);
        }
    }

    #[test]
    fn duplicate_import_response_uses_desktop_field_names() {
        let task_id = TaskId::generate();
        let response = serde_json::to_value(ExternalWorkItemImportResponse::Existing { task_id })
            .expect("serialize duplicate import response");
        assert_eq!(response["outcome"], "existing");
        assert_eq!(response["taskId"], task_id.to_string());
        assert!(response.get("task_id").is_none());
    }

    #[test]
    fn runtime_work_item_capabilities_stay_within_catalog() {
        let catalog = admitted_work_item_provider("github").expect("GitHub catalog provider");
        let mut extra = catalog.plugin_descriptor();
        extra.capabilities.push("work_item.unadmitted".into());
        let runtime = WorkItemProviderDescriptor::from_plugin_descriptor(&extra)
            .expect("runtime descriptor shape");
        assert!(negotiate_work_item_provider(&catalog, runtime, &extra.schema_versions).is_err());

        let mut incompatible = catalog.plugin_descriptor();
        incompatible
            .schema_versions
            .insert("plugin".into(), "v2".into());
        let runtime = WorkItemProviderDescriptor::from_plugin_descriptor(&incompatible)
            .expect("schema-mismatched descriptor shape");
        assert!(
            negotiate_work_item_provider(&catalog, runtime, &incompatible.schema_versions).is_err()
        );

        let mut reduced = catalog.plugin_descriptor();
        reduced
            .capabilities
            .retain(|capability| capability != "work_item.resolve");
        let runtime = WorkItemProviderDescriptor::from_plugin_descriptor(&reduced)
            .expect("reduced runtime descriptor shape");
        assert!(negotiate_work_item_provider(&catalog, runtime, &reduced.schema_versions).is_err());
    }

    #[test]
    fn agent_definitions_are_served_by_core() {
        let definitions = agent_defs_list();
        assert_eq!(definitions.len(), 6);
        let builder = agent_def("builder".into()).expect("builder seed definition");
        assert_eq!(builder.name, "builder");
        assert_eq!(builder.frontmatter["task_class"], "code");
    }

    #[test]
    fn ipc_errors_expose_a_machine_readable_kind() {
        let error = agent_def("missing".into()).expect_err("missing definition is refused");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }

    #[test]
    fn linters_count_is_served_by_core() {
        let root = std::env::temp_dir().join(format!(
            "locus-linter-count-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after Unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create linter directory");
        std::fs::write(root.join("format.sh"), "exit 0").expect("write check");
        std::fs::write(root.join("format.md"), "Use the project formatter.").expect("write rule");
        assert_eq!(
            linter_count(root.display().to_string()).expect("count linters"),
            1
        );
        std::fs::remove_dir_all(root).expect("remove linter directory");
    }

    #[test]
    fn materialization_report_is_derived_from_the_core_registry() {
        // Reads the registry the composition root parsed at start, not the disk.
        let core = Core::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(HARNESS_REGISTRY))
            .expect("core loads");
        let reports = reports_for_registry(core.registry());
        assert!(!reports.is_empty());
        assert!(reports.iter().all(|report| !report.harness.is_empty()));
        assert!(reports
            .iter()
            .flat_map(|report| &report.losses)
            .all(|loss| !loss.weaker_than_native.is_empty()));
    }

    #[test]
    fn harness_tier_grid_preserves_free_text_and_unset_tiers() {
        let response = harness_tier_grid(HarnessTierGridRequest {
            project_id: "project-1".into(),
            harnesses: vec![HarnessTierGridHarnessRequest {
                name: "claude".into(),
                models: None,
            }],
            tier_settings: vec![ModelTierSetting {
                harness: "claude".into(),
                tier: "high".into(),
                model: Some("opus".into()),
            }],
        })
        .expect("shape settings grid");
        assert_eq!(response.harnesses[0].models, None);
        assert_eq!(response.harnesses[0].tiers.len(), MODEL_TIERS.len());
        assert_eq!(
            response.harnesses[0].tiers[2].model.as_deref(),
            Some("opus")
        );
        assert_eq!(response.harnesses[0].tiers[3].model, None);
    }
}
