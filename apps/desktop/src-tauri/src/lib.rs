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

/// The Setup screen's live read path: projects, their repos and local remotes, and
/// the per-project harness policy and base context. The `*_inner` fns are the test
/// surface — a Tauri `State` cannot be constructed outside the runtime.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSummaryResponse {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoResponse {
    id: String,
    project_id: String,
    name: String,
    working_copy_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalRemoteResponse {
    id: String,
    repo_id: String,
    bare_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSetupResponse {
    harness_allow_list: Vec<String>,
    base_context: Option<String>,
    base_context_token_budget: Option<u32>,
}

async fn projects_list_inner(store: &Store) -> Result<Vec<ProjectSummaryResponse>, IpcError> {
    store
        .projects_list()
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| ProjectSummaryResponse {
                    id: row.id.to_string(),
                    name: row.name,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

/// A Setup read is scoped to a project that exists: an unknown id is a typed
/// not-found, never an empty success pretending the project has nothing.
async fn resolve_setup_project(store: &Store, identifier: &str) -> Result<ProjectId, IpcError> {
    store
        .resolve_project_id(identifier)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("project was not found"))
}

async fn repos_list_inner(store: &Store, project_id: &str) -> Result<Vec<RepoResponse>, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    store
        .repos_list(project_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| RepoResponse {
                    id: row.id.to_string(),
                    project_id: row.project_id.to_string(),
                    name: row.name,
                    working_copy_path: row.working_copy_path,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn local_remotes_list_inner(
    store: &Store,
    project_id: &str,
) -> Result<Vec<LocalRemoteResponse>, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    store
        .local_remotes_list(project_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| LocalRemoteResponse {
                    id: row.id.to_string(),
                    repo_id: row.repo_id.to_string(),
                    bare_path: row.bare_path,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn project_setup_inner(
    store: &Store,
    project_id: &str,
) -> Result<ProjectSetupResponse, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    store
        .project_settings(project_id)
        .await
        .map(|settings| ProjectSetupResponse {
            harness_allow_list: settings.harness_allow_list().to_vec(),
            base_context: settings.base_context().map(str::to_owned),
            base_context_token_budget: settings.base_context_token_budget(),
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn projects_list(
    core: State<'_, Arc<Core>>,
) -> Result<Vec<ProjectSummaryResponse>, IpcError> {
    let store = connected_store(&core).await?;
    projects_list_inner(store).await
}

#[tauri::command]
async fn repos_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<RepoResponse>, IpcError> {
    let store = connected_store(&core).await?;
    repos_list_inner(store, &project_id).await
}

#[tauri::command]
async fn local_remotes_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<LocalRemoteResponse>, IpcError> {
    let store = connected_store(&core).await?;
    local_remotes_list_inner(store, &project_id).await
}

#[tauri::command]
async fn project_setup(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<ProjectSetupResponse, IpcError> {
    let store = connected_store(&core).await?;
    project_setup_inner(store, &project_id).await
}

/// The shell's live pill data (slice 4): dispatch running counts and sessions,
/// and the Inbox pill's pending-for-a-human count.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StripCardResponse {
    id: String,
    project: String,
    agent: String,
    status: String,
    /// Seconds since the run started, so the UI derives elapsed time locally.
    started_epoch: i64,
}

async fn scope_project(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Option<ProjectId>, IpcError> {
    match project_id {
        None => Ok(None),
        Some(identifier) => Ok(Some(
            store
                .resolve_project_id(identifier)
                .await
                .map_err(IpcError::internal)?
                .ok_or_else(|| IpcError::not_found("project was not found"))?,
        )),
    }
}

async fn strip_cards_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<StripCardResponse>, IpcError> {
    let project_id = scope_project(store, project_id).await?;
    store
        .running_runs(project_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| StripCardResponse {
                    id: row.id.to_string(),
                    project: row.project,
                    agent: row.agent,
                    status: row.status,
                    started_epoch: row.started_epoch,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn running_count_inner(store: &Store, project_id: Option<&str>) -> Result<usize, IpcError> {
    let project_id = scope_project(store, project_id).await?;
    let count = store
        .running_run_count(project_id)
        .await
        .map_err(IpcError::internal)?;
    usize::try_from(count).map_err(|_| IpcError::internal("run count exceeds usize"))
}

async fn inbox_pending_count_inner(store: &Store) -> Result<usize, IpcError> {
    let count = store
        .pending_human_delivery_count()
        .await
        .map_err(IpcError::internal)?;
    usize::try_from(count).map_err(|_| IpcError::internal("inbox count exceeds usize"))
}

#[tauri::command]
async fn strip_cards(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<StripCardResponse>, IpcError> {
    let store = connected_store(&core).await?;
    strip_cards_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn running_count(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<usize, IpcError> {
    let store = connected_store(&core).await?;
    running_count_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn inbox_pending_count(core: State<'_, Arc<Core>>) -> Result<usize, IpcError> {
    let store = connected_store(&core).await?;
    inbox_pending_count_inner(store).await
}

/// The Dispatch runs table (slice 7): every run across projects, newest first,
/// with event and error rollups.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DispatchRunResponse {
    id: String,
    project: String,
    agent: String,
    branch: String,
    status: String,
    harness: Option<String>,
    role: Option<String>,
    model: String,
    events: i64,
    errors: i64,
    started_at: Option<String>,
}

async fn dispatch_runs_page_inner(
    store: &Store,
    project_id: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<DispatchRunResponse>, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    let offset = i64::try_from(offset.max(0) as u64).unwrap_or(0);
    let limit = i64::try_from(limit.clamp(0, 500) as u64).unwrap_or(100);
    store
        .dispatch_runs_page(scoped, offset, limit)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| DispatchRunResponse {
                    id: row.id.to_string(),
                    project: row.project,
                    agent: row.agent,
                    branch: row.branch,
                    status: row.status,
                    harness: row.harness,
                    role: row.role,
                    model: row.model,
                    events: row.events,
                    errors: row.errors,
                    started_at: row.started_at,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn dispatch_runs_count_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<usize, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    let count = store
        .dispatch_runs_count(scoped)
        .await
        .map_err(IpcError::internal)?;
    usize::try_from(count).map_err(|_| IpcError::internal("run count exceeds usize"))
}

#[tauri::command]
async fn dispatch_runs_page(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<DispatchRunResponse>, IpcError> {
    let store = connected_store(&core).await?;
    dispatch_runs_page_inner(
        store,
        project_id.as_deref(),
        offset.unwrap_or(0),
        limit.unwrap_or(100),
    )
    .await
}

#[tauri::command]
async fn dispatch_runs_count(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<usize, IpcError> {
    let store = connected_store(&core).await?;
    dispatch_runs_count_inner(store, project_id.as_deref()).await
}

/// The sessions family (slice 7): the session list and one session's runs.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    id: String,
    project_id: String,
    project: String,
    agent: String,
    name: String,
    branch: String,
    status: String,
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRunResponse {
    id: String,
    session_id: String,
    status: String,
    resolved_model: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    exit_code: Option<i32>,
}

async fn sessions_list_inner(
    store: &Store,
    project_id: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<SessionResponse>, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    store
        .sessions_page(scoped, offset.max(0), limit.clamp(0, 500))
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| SessionResponse {
                    id: row.id.to_string(),
                    project_id: row.project_id.to_string(),
                    project: row.project,
                    agent: row.agent,
                    name: row.name,
                    branch: row.branch,
                    status: row.status,
                    created_at: row.created_at,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn runs_for_session_inner(
    store: &Store,
    session_id: &str,
) -> Result<Vec<SessionRunResponse>, IpcError> {
    let session_id: uuid::Uuid = session_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("session id must be a UUID"))?;
    store
        .runs_for_session(session_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| SessionRunResponse {
                    id: row.id.to_string(),
                    session_id: row.session_id.to_string(),
                    status: row.status,
                    resolved_model: row.resolved_model,
                    started_at: row.started_at,
                    ended_at: row.ended_at,
                    exit_code: row.exit_code,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn sessions_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<SessionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    sessions_list_inner(
        store,
        project_id.as_deref(),
        offset.unwrap_or(0),
        limit.unwrap_or(100),
    )
    .await
}

#[tauri::command]
async fn runs_for_session(
    core: State<'_, Arc<Core>>,
    session_id: String,
) -> Result<Vec<SessionRunResponse>, IpcError> {
    let store = connected_store(&core).await?;
    runs_for_session_inner(store, &session_id).await
}

/// Setup's settings mutations (slice 5): base context, archive, rename.
async fn project_base_context_set_inner(
    store: &Store,
    project_id: &str,
    content: &str,
    token_budget: Option<u32>,
) -> Result<ProjectSetupResponse, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    // The domain rule: base context and a nonzero budget rise and fall together.
    // Empty content therefore clears both; non-empty content requires a budget.
    let content = content.trim();
    let (context_value, budget_value) = if content.is_empty() {
        if token_budget.is_some() {
            return Err(IpcError::invalid_argument(
                "clearing the base context also clears its token budget",
            ));
        }
        (serde_json::Value::Null, serde_json::Value::Null)
    } else {
        let budget = token_budget.ok_or_else(|| {
            IpcError::invalid_argument("a base context needs a nonzero token budget")
        })?;
        if budget == 0 {
            return Err(IpcError::invalid_argument(
                "a base context needs a nonzero token budget",
            ));
        }
        (
            serde_json::Value::String(content.to_owned()),
            serde_json::Value::Number(budget.into()),
        )
    };
    let settings = store
        .project_settings(project_id)
        .await
        .map_err(IpcError::internal)?;
    // ProjectSettings keeps its policy fields private; the round trip through the
    // serialized form updates exactly the two base-context keys.
    let mut value = serde_json::to_value(&settings).map_err(IpcError::internal)?;
    value["base_context"] = context_value;
    value["base_context_token_budget"] = budget_value;
    let updated: locus_core::services::project::ProjectSettings =
        serde_json::from_value(value).map_err(IpcError::internal)?;
    store
        .set_project_settings(project_id, &updated)
        .await
        .map_err(IpcError::internal)?;
    project_setup_inner(store, project_id.to_string().as_str()).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectArchiveResponse {
    archived: bool,
}

async fn project_archive_set_inner(
    store: &Store,
    project_id: &str,
    archived: bool,
) -> Result<ProjectArchiveResponse, IpcError> {
    let pid = resolve_setup_project(store, project_id).await?;
    store
        .set_project_archived(pid, archived)
        .await
        .map_err(IpcError::internal)?;
    Ok(ProjectArchiveResponse { archived })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRenameResponse {
    id: String,
    name: String,
}

async fn project_rename_inner(
    store: &Store,
    project_id: &str,
    name: &str,
) -> Result<ProjectRenameResponse, IpcError> {
    let pid = resolve_setup_project(store, project_id).await?;
    if name.trim().is_empty() {
        return Err(IpcError::invalid_argument("project name must not be empty"));
    }
    store
        .rename_project(pid, name)
        .await
        .map_err(IpcError::internal)?;
    Ok(ProjectRenameResponse {
        id: pid.to_string(),
        name: name.to_owned(),
    })
}

#[tauri::command]
async fn project_base_context_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    content: String,
    token_budget: Option<u32>,
) -> Result<ProjectSetupResponse, IpcError> {
    let store = connected_store(&core).await?;
    project_base_context_set_inner(store, &project_id, &content, token_budget).await
}

#[tauri::command]
async fn project_archive_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    archived: bool,
) -> Result<ProjectArchiveResponse, IpcError> {
    let store = connected_store(&core).await?;
    project_archive_set_inner(store, &project_id, archived).await
}

#[tauri::command]
async fn project_rename(
    core: State<'_, Arc<Core>>,
    project_id: String,
    name: String,
) -> Result<ProjectRenameResponse, IpcError> {
    let store = connected_store(&core).await?;
    project_rename_inner(store, &project_id, &name).await
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
            projects_list,
            repos_list,
            local_remotes_list,
            project_setup,
            project_base_context_set,
            project_archive_set,
            project_rename,
            strip_cards,
            running_count,
            inbox_pending_count,
            dispatch_runs_page,
            dispatch_runs_count,
            sessions_list,
            runs_for_session,
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

/// The tracer bullet: Setup reads projects, repos, local remotes, harness policy,
/// and base context from a real store. Two projects are seeded so every read is
/// also proven project-scoped.
#[cfg(test)]
mod setup_live_data {
    use super::*;
    use locus_core::services::project::ProjectSettings;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    fn expect_not_found<T: std::fmt::Debug>(read: Result<T, IpcError>) {
        let error = read.expect_err("an unknown project is a typed not-found");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-setup-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the setup test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the setup test store");
        (store, cleanup)
    }

    async fn seed_project(store: &Store, id: &str, name: &str) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, $2)")
            .bind(id)
            .bind(name)
            .execute(store.test_pool())
            .await
            .expect("seed project");
    }

    async fn seed_repo(store: &Store, id: &str, project_id: &str, name: &str, path: &str) {
        sqlx::query(
            "INSERT INTO core.repos (id, project_id, name, working_copy_path)
             VALUES ($1::uuid, $2::uuid, $3, $4)",
        )
        .bind(id)
        .bind(project_id)
        .bind(name)
        .bind(path)
        .execute(store.test_pool())
        .await
        .expect("seed repo");
    }

    #[tokio::test]
    async fn lists_projects_alphabetically() {
        let (store, _cleanup) = test_store().await;
        seed_project(&store, "00000000-0000-0000-0000-000000000302", "weaver").await;
        seed_project(&store, "00000000-0000-0000-0000-000000000301", "amq").await;

        let projects = projects_list_inner(&store).await.expect("list projects");
        let names: Vec<&str> = projects
            .iter()
            .map(|project| project.name.as_str())
            .collect();
        assert_eq!(names, ["amq", "weaver"]);
        assert_eq!(projects[0].id, "00000000-0000-0000-0000-000000000301");
    }

    #[tokio::test]
    async fn repos_never_cross_the_project_boundary() {
        let (store, _cleanup) = test_store().await;
        let tapestry = "00000000-0000-0000-0000-000000000301";
        let loom = "00000000-0000-0000-0000-000000000302";
        seed_project(&store, tapestry, "tapestry").await;
        seed_project(&store, loom, "loom-db").await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000311",
            tapestry,
            "core",
            "/checkouts/tapestry-core",
        )
        .await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000312",
            tapestry,
            "desktop",
            "/checkouts/tapestry-desktop",
        )
        .await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000321",
            loom,
            "loom",
            "/checkouts/loom",
        )
        .await;

        let tapestry_repos = repos_list_inner(&store, tapestry)
            .await
            .expect("list tapestry repos");
        assert_eq!(tapestry_repos.len(), 2);
        assert!(tapestry_repos
            .iter()
            .all(|repo| repo.project_id == tapestry));

        let loom_repos = repos_list_inner(&store, loom)
            .await
            .expect("list loom repos");
        assert_eq!(loom_repos.len(), 1);
        assert_eq!(loom_repos[0].name, "loom");
        assert_eq!(loom_repos[0].working_copy_path, "/checkouts/loom");
    }

    #[tokio::test]
    async fn unknown_project_is_rejected_not_emptied() {
        let (store, _cleanup) = test_store().await;
        seed_project(&store, "00000000-0000-0000-0000-000000000301", "tapestry").await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000311",
            "00000000-0000-0000-0000-000000000301",
            "core",
            "/checkouts/core",
        )
        .await;

        expect_not_found(repos_list_inner(&store, "00000000-0000-0000-0000-0000000003ff").await);
        expect_not_found(
            local_remotes_list_inner(&store, "00000000-0000-0000-0000-0000000003ff").await,
        );
        expect_not_found(project_setup_inner(&store, "00000000-0000-0000-0000-0000000003ff").await);
    }

    #[tokio::test]
    async fn setup_reads_harness_policy_and_base_context() {
        let (store, _cleanup) = test_store().await;
        let tapestry = "00000000-0000-0000-0000-000000000301";
        seed_project(&store, tapestry, "tapestry").await;
        let settings: ProjectSettings = serde_json::from_value(serde_json::json!({
            "version": 1,
            "harness_allow_list": ["claude", "codex"],
            "base_context": "# Working in tapestry\n\nYour branch is never main.",
            "base_context_token_budget": 1500,
        }))
        .expect("shape seeded settings");
        store
            .set_project_settings(tapestry.parse().expect("seed project id"), &settings)
            .await
            .expect("seed settings");

        let setup = project_setup_inner(&store, tapestry)
            .await
            .expect("read project setup");
        assert_eq!(setup.harness_allow_list, ["claude", "codex"]);
        assert_eq!(
            setup.base_context.as_deref(),
            Some("# Working in tapestry\n\nYour branch is never main.")
        );
        assert_eq!(setup.base_context_token_budget, Some(1500));
    }

    #[tokio::test]
    async fn setup_defaults_when_no_policy_is_stored() {
        let (store, _cleanup) = test_store().await;
        seed_project(&store, "00000000-0000-0000-0000-000000000301", "tapestry").await;

        let setup = project_setup_inner(&store, "00000000-0000-0000-0000-000000000301")
            .await
            .expect("read default setup");
        assert!(setup.harness_allow_list.is_empty());
        assert_eq!(setup.base_context, None);
        assert_eq!(setup.base_context_token_budget, None);
    }

    #[tokio::test]
    async fn local_remotes_scope_through_their_repo() {
        let (store, _cleanup) = test_store().await;
        let tapestry = "00000000-0000-0000-0000-000000000301";
        let loom = "00000000-0000-0000-0000-000000000302";
        seed_project(&store, tapestry, "tapestry").await;
        seed_project(&store, loom, "loom-db").await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000311",
            tapestry,
            "core",
            "/checkouts/tapestry-core",
        )
        .await;
        seed_repo(
            &store,
            "00000000-0000-0000-0000-000000000321",
            loom,
            "loom",
            "/checkouts/loom",
        )
        .await;
        sqlx::query(
            "INSERT INTO core.local_remotes (id, repo_id, bare_path)
             VALUES ('00000000-0000-0000-0000-000000000331', '00000000-0000-0000-0000-000000000311', '/var/lib/locus/repos/tapestry-core.git'),
                    ('00000000-0000-0000-0000-000000000332', '00000000-0000-0000-0000-000000000321', '/var/lib/locus/repos/loom.git')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed local remotes");

        let tapestry_remotes = local_remotes_list_inner(&store, tapestry)
            .await
            .expect("list tapestry remotes");
        assert_eq!(tapestry_remotes.len(), 1);
        assert_eq!(
            tapestry_remotes[0].bare_path,
            "/var/lib/locus/repos/tapestry-core.git"
        );
        assert_eq!(
            tapestry_remotes[0].repo_id,
            "00000000-0000-0000-0000-000000000311"
        );
    }
}

/// Shell pill queries: the dispatch pill counts and lists running runs; the Inbox
/// pill counts human-pending deliveries.
#[cfg(test)]
mod shell_queries {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-shell-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the shell test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the shell test store");
        (store, cleanup)
    }

    async fn seed_running_run(
        store: &Store,
        project_id: &str,
        project_name: &str,
        agent_def_id: &str,
        agent_name: &str,
        session_id: &str,
        run_id: &str,
        status: &str,
    ) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, $2)")
            .bind(project_id)
            .bind(project_name)
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, $2, 1, '{}'::jsonb, 'test agent')",
        )
        .bind(agent_def_id)
        .bind(agent_name)
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'shell session', 'agent/shell')",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(agent_def_id)
        .execute(store.test_pool())
        .await
        .expect("seed session");
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ($1::uuid, $2::uuid, 'test-model', $3, now())",
        )
        .bind(run_id)
        .bind(session_id)
        .bind(status)
        .execute(store.test_pool())
        .await
        .expect("seed run");
    }

    #[tokio::test]
    async fn running_count_and_cards_agree() {
        let (store, _cleanup) = test_store().await;
        seed_running_run(
            &store,
            "00000000-0000-0000-0000-000000000401",
            "tapestry",
            "00000000-0000-0000-0000-000000000411",
            "builder",
            "00000000-0000-0000-0000-000000000421",
            "00000000-0000-0000-0000-000000000431",
            "running",
        )
        .await;
        let count = running_count_inner(&store, None)
            .await
            .expect("running count");
        assert_eq!(count, 1);
        let cards = strip_cards_inner(&store, None).await.expect("strip cards");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].project, "tapestry");
        assert_eq!(cards[0].agent, "builder");
        assert_eq!(cards[0].status, "running");
    }

    #[tokio::test]
    async fn only_running_runs_count() {
        let (store, _cleanup) = test_store().await;
        seed_running_run(
            &store,
            "00000000-0000-0000-0000-000000000401",
            "tapestry",
            "00000000-0000-0000-0000-000000000411",
            "builder",
            "00000000-0000-0000-0000-000000000421",
            "00000000-0000-0000-0000-000000000431",
            "completed",
        )
        .await;
        let count = running_count_inner(&store, None)
            .await
            .expect("running count");
        assert_eq!(count, 0);
        assert!(strip_cards_inner(&store, None)
            .await
            .expect("strip cards")
            .is_empty());
    }

    #[tokio::test]
    async fn the_inbox_pill_counts_only_human_pending_deliveries() {
        let (store, _cleanup) = test_store().await;
        // Seed the chain a human delivery hangs off: thread → message → delivery.
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000401', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000411', 'builder', 1, '{}'::jsonb, 'test agent')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000421', '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000411', 'shell session', 'agent/shell')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed session");
        sqlx::query(
            "INSERT INTO mail.threads (id, project_id, subject)
             VALUES ('00000000-0000-0000-0000-000000000441', '00000000-0000-0000-0000-000000000401', 'gate')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed thread");
        sqlx::query(
            "INSERT INTO mail.messages (id, thread_id, sender_kind, body)
             VALUES ('00000000-0000-0000-0000-000000000451', '00000000-0000-0000-0000-000000000441', 'agent', 'needs a human')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed message");
        sqlx::query(
            "INSERT INTO mail.deliveries (id, message_id, recipient_kind, recipient_session_id, status)
             VALUES ('00000000-0000-0000-0000-000000000461', '00000000-0000-0000-0000-000000000451', 'human', NULL, 'pending'),
                    ('00000000-0000-0000-0000-000000000462', '00000000-0000-0000-0000-000000000451', 'agent', '00000000-0000-0000-0000-000000000421', 'pending')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed deliveries");

        let count = inbox_pending_count_inner(&store)
            .await
            .expect("inbox pending count");
        // The agent-addressed pending delivery is not the Inbox's business.
        assert_eq!(count, 1);
    }
}

/// Shell scope: the same queries accept a project filter and never leak another
/// project's rows into it.
#[cfg(test)]
mod shell_scope {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-shell-scope").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the shell scope test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the shell scope test store");
        (store, cleanup)
    }

    async fn seed_run(
        store: &Store,
        project_id: &str,
        project_name: &str,
        agent_def_id: &str,
        session_id: &str,
        run_id: &str,
    ) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, $2)")
            .bind(project_id)
            .bind(project_name)
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, $2, 1, '{}'::jsonb, 'test agent')",
        )
        .bind(agent_def_id)
        .bind(project_name)
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'shell session', 'agent/shell')",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(agent_def_id)
        .execute(store.test_pool())
        .await
        .expect("seed session");
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ($1::uuid, $2::uuid, 'test-model', 'running', now())",
        )
        .bind(run_id)
        .bind(session_id)
        .execute(store.test_pool())
        .await
        .expect("seed run");
    }

    #[tokio::test]
    async fn a_scoped_read_excludes_other_projects() {
        let (store, _cleanup) = test_store().await;
        let tapestry = "00000000-0000-0000-0000-000000000401";
        let loom = "00000000-0000-0000-0000-000000000402";
        seed_run(
            &store,
            tapestry,
            "tapestry",
            "00000000-0000-0000-0000-000000000411",
            "00000000-0000-0000-0000-000000000421",
            "00000000-0000-0000-0000-000000000431",
        )
        .await;
        seed_run(
            &store,
            loom,
            "loom-db",
            "00000000-0000-0000-0000-000000000412",
            "00000000-0000-0000-0000-000000000422",
            "00000000-0000-0000-0000-000000000432",
        )
        .await;

        let tapestry_cards = strip_cards_inner(&store, Some(tapestry))
            .await
            .expect("scoped strip cards");
        assert_eq!(tapestry_cards.len(), 1);
        assert_eq!(tapestry_cards[0].project, "tapestry");

        let tapestry_count = running_count_inner(&store, Some(tapestry))
            .await
            .expect("scoped running count");
        assert_eq!(tapestry_count, 1);

        // The unscoped shell view still sees both.
        assert_eq!(running_count_inner(&store, None).await.expect("count"), 2);
    }

    #[tokio::test]
    async fn an_unknown_scope_is_rejected_not_emptied() {
        let (store, _cleanup) = test_store().await;
        let error = strip_cards_inner(&store, Some("00000000-0000-0000-0000-0000000004ff"))
            .await
            .expect_err("an unknown scope is a typed not-found");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }
}

/// Shell mutations: Setup's base-context save, archive, and rename hit the real
/// store, and every failure is a typed rejection.
#[cfg(test)]
mod shell_mutations {
    use super::*;
    use locus_core::services::project::ProjectSettings;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-mutations").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the mutation test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the mutation test store");
        (store, cleanup)
    }

    const TAPESTRY: &str = "00000000-0000-0000-0000-000000000501";

    async fn seed_tapestry(store: &Store) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'tapestry')")
            .bind(TAPESTRY)
            .execute(store.test_pool())
            .await
            .expect("seed project");
    }

    #[tokio::test]
    async fn base_context_set_persists_content_and_budget() {
        let (store, _cleanup) = test_store().await;
        seed_tapestry(&store).await;

        let setup = project_base_context_set_inner(
            &store,
            TAPESTRY,
            "# Working in tapestry\n\nVerify with cargo test.",
            Some(1200),
        )
        .await
        .expect("save base context");
        assert_eq!(
            setup.base_context.as_deref(),
            Some("# Working in tapestry\n\nVerify with cargo test.")
        );
        assert_eq!(setup.base_context_token_budget, Some(1200));

        // It is durable: a fresh read sees it, and unrelated policy survives.
        let reread = project_setup_inner(&store, TAPESTRY).await.expect("reread");
        assert_eq!(reread.base_context_token_budget, Some(1200));
    }

    #[tokio::test]
    async fn base_context_clears_both_sides_of_the_domain_rule() {
        let (store, _cleanup) = test_store().await;
        seed_tapestry(&store).await;

        let cleared = project_base_context_set_inner(&store, TAPESTRY, "", None)
            .await
            .expect("clear base context");
        assert_eq!(cleared.base_context, None);
        assert_eq!(cleared.base_context_token_budget, None);

        // Content without a budget breaks the together-rule and is refused.
        let error = project_base_context_set_inner(&store, TAPESTRY, "# kept", None)
            .await
            .expect_err("content needs a budget");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "invalid_argument"
        );
    }

    #[tokio::test]
    async fn archive_and_rename_persist() {
        let (store, _cleanup) = test_store().await;
        seed_tapestry(&store).await;

        let archived = project_archive_set_inner(&store, TAPESTRY, true)
            .await
            .expect("archive project");
        assert!(archived.archived);
        assert!(store
            .project_archived(TAPESTRY.parse().expect("id"))
            .await
            .expect("read archived"));

        let renamed = project_rename_inner(&store, TAPESTRY, "weaver")
            .await
            .expect("rename project");
        assert_eq!(renamed.name, "weaver");
        let projects = projects_list_inner(&store).await.expect("list projects");
        assert_eq!(projects[0].name, "weaver");
    }

    #[tokio::test]
    async fn mutations_reject_unknown_projects_and_empty_names() {
        let (store, _cleanup) = test_store().await;
        seed_tapestry(&store).await;
        let missing = "00000000-0000-0000-0000-0000000005ff";

        for error in [
            project_base_context_set_inner(&store, missing, "x", None)
                .await
                .expect_err("unknown project rejected"),
            project_archive_set_inner(&store, missing, true)
                .await
                .expect_err("unknown project rejected"),
            project_rename_inner(&store, missing, "weaver")
                .await
                .expect_err("unknown project rejected"),
        ] {
            assert_eq!(
                serde_json::to_value(error).expect("serialize IPC error")["kind"],
                "not_found"
            );
        }

        let empty = project_rename_inner(&store, TAPESTRY, "   ")
            .await
            .expect_err("an empty name is refused");
        assert_eq!(
            serde_json::to_value(empty).expect("serialize IPC error")["kind"],
            "invalid_argument"
        );
    }

    #[tokio::test]
    async fn base_context_set_keeps_other_policy_fields() {
        let (store, _cleanup) = test_store().await;
        seed_tapestry(&store).await;
        let policy: ProjectSettings = serde_json::from_value(serde_json::json!({
            "version": 1,
            "harness_allow_list": ["claude"],
        }))
        .expect("shape policy");
        store
            .set_project_settings(TAPESTRY.parse().expect("id"), &policy)
            .await
            .expect("seed policy");

        let setup = project_base_context_set_inner(&store, TAPESTRY, "# kept", Some(900))
            .await
            .expect("save base context");
        // The round trip through the serialized settings preserved the allow list.
        assert_eq!(setup.harness_allow_list, ["claude"]);
        assert_eq!(setup.base_context.as_deref(), Some("# kept"));
    }
}

/// Run queries: the Dispatch runs table reads every run with its event rollups,
/// scoped by project, ordered newest first.
#[cfg(test)]
mod run_queries {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-run-queries").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the run query test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the run query test store");
        (store, cleanup)
    }

    #[tokio::test]
    async fn pages_runs_newest_first_with_event_rollups() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000601', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000611', 'builder', 1, '{\"harness\": \"claude\", \"role\": \"builder\"}'::jsonb, 'test agent')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000621', '00000000-0000-0000-0000-000000000601', '00000000-0000-0000-0000-000000000611', 'run query session', 'agent/run-queries')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed session");
        // Two runs, an hour apart: the newer one must page first.
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000631', '00000000-0000-0000-0000-000000000621', 'claude-opus-4', 'completed', now() - interval '1 hour'),
                    ('00000000-0000-0000-0000-000000000632', '00000000-0000-0000-0000-000000000621', 'claude-opus-4', 'running', now())",
        )
        .execute(store.test_pool())
        .await
        .expect("seed runs");
        sqlx::query(
            "INSERT INTO agents.events (id, run_id, seq, ts, verb, raw)
             VALUES ('00000000-0000-0000-0000-000000000641', '00000000-0000-0000-0000-000000000632', 0, now(), 'assistant', '{}'::jsonb),
                    ('00000000-0000-0000-0000-000000000642', '00000000-0000-0000-0000-000000000632', 1, now(), 'tool_call', '{}'::jsonb),
                    ('00000000-0000-0000-0000-000000000643', '00000000-0000-0000-0000-000000000632', 2, now(), 'tool_error', '{}'::jsonb)",
        )
        .execute(store.test_pool())
        .await
        .expect("seed events");

        let page = dispatch_runs_page_inner(&store, None, 0, 100)
            .await
            .expect("page runs");
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].id, "00000000-0000-0000-0000-000000000632");
        assert_eq!(page[0].project, "tapestry");
        assert_eq!(page[0].harness.as_deref(), Some("claude"));
        assert_eq!(page[0].role.as_deref(), Some("builder"));
        assert_eq!(page[0].events, 3);
        assert_eq!(page[0].errors, 1);
        // The older run has no events: unknown would be a lie here — zero is the
        // truth the rollup knows.
        assert_eq!(page[1].events, 0);

        let count = dispatch_runs_count_inner(&store, None)
            .await
            .expect("count runs");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn a_project_scope_excludes_other_projects_and_unknown_rejects() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000601', 'tapestry'), ('00000000-0000-0000-0000-000000000602', 'loom-db')")
            .execute(store.test_pool())
            .await
            .expect("seed projects");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000611', 'builder', 1, '{}'::jsonb, 'test agent')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000621', '00000000-0000-0000-0000-000000000601', '00000000-0000-0000-0000-000000000611', 'a', 'agent/a'),
                    ('00000000-0000-0000-0000-000000000622', '00000000-0000-0000-0000-000000000602', '00000000-0000-0000-0000-000000000611', 'b', 'agent/b')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed sessions");
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000631', '00000000-0000-0000-0000-000000000621', 'm', 'running', now()),
                    ('00000000-0000-0000-0000-000000000632', '00000000-0000-0000-0000-000000000622', 'm', 'running', now())",
        )
        .execute(store.test_pool())
        .await
        .expect("seed runs");

        let scoped =
            dispatch_runs_page_inner(&store, Some("00000000-0000-0000-0000-000000000601"), 0, 100)
                .await
                .expect("scoped page");
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].project, "tapestry");

        let error =
            dispatch_runs_page_inner(&store, Some("00000000-0000-0000-0000-0000000006ff"), 0, 100)
                .await
                .expect_err("unknown scope rejected");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }
}


/// Session queries: the session list reads every session with its project and
/// agent, scoped by project, and a session's runs read oldest first.
#[cfg(test)]
mod session_queries {
    use super::*;
    use uuid::Uuid;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-sessions").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the session test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the session test store");
        (store, cleanup)
    }

    async fn seed_session(
        store: &Store,
        project_id: &str,
        project_name: &str,
        agent_def_id: &str,
        agent_name: &str,
        session_id: &str,
        branch: &str,
    ) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, $2)")
            .bind(project_id)
            .bind(project_name)
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, $2, 1, '{}'::jsonb, 'test agent')",
        )
        .bind(agent_def_id)
        .bind(agent_name)
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'session body', $4)",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(agent_def_id)
        .bind(branch)
        .execute(store.test_pool())
        .await
        .expect("seed session");
    }

    #[tokio::test]
    async fn lists_sessions_with_project_and_agent_resolved() {
        let (store, _cleanup) = test_store().await;
        seed_session(
            &store,
            "00000000-0000-0000-0000-000000000701",
            "tapestry",
            "00000000-0000-0000-0000-000000000711",
            "builder",
            "00000000-0000-0000-0000-000000000721",
            "agent/tapestry",
        )
        .await;

        let sessions = sessions_list_inner(&store, None, 0, 100)
            .await
            .expect("list sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].project, "tapestry");
        assert_eq!(sessions[0].agent, "builder");
        assert_eq!(sessions[0].branch, "agent/tapestry");
        assert_eq!(sessions[0].status, "active");
    }

    #[tokio::test]
    async fn a_project_scope_excludes_other_projects() {
        let (store, _cleanup) = test_store().await;
        seed_session(
            &store,
            "00000000-0000-0000-0000-000000000701",
            "tapestry",
            "00000000-0000-0000-0000-000000000711",
            "builder",
            "00000000-0000-0000-0000-000000000721",
            "agent/tapestry",
        )
        .await;
        seed_session(
            &store,
            "00000000-0000-0000-0000-000000000702",
            "loom-db",
            "00000000-0000-0000-0000-000000000712",
            "reviewer",
            "00000000-0000-0000-0000-000000000722",
            "agent/loom",
        )
        .await;

        let scoped = sessions_list_inner(
            &store,
            Some("00000000-0000-0000-0000-000000000701"),
            0,
            100,
        )
        .await
        .expect("scoped sessions");
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].project, "tapestry");

        let error = sessions_list_inner(
            &store,
            Some("00000000-0000-0000-0000-0000000007ff"),
            0,
            100,
        )
        .await
        .expect_err("unknown scope rejected");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }

    #[tokio::test]
    async fn a_sessions_runs_read_oldest_first_and_rejects_bad_ids() {
        let (store, _cleanup) = test_store().await;
        seed_session(
            &store,
            "00000000-0000-0000-0000-000000000701",
            "tapestry",
            "00000000-0000-0000-0000-000000000711",
            "builder",
            "00000000-0000-0000-0000-000000000721",
            "agent/tapestry",
        )
        .await;
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000731', '00000000-0000-0000-0000-000000000721', 'claude-opus-4', 'completed', now() - interval '2 hours'),
                    ('00000000-0000-0000-0000-000000000732', '00000000-0000-0000-0000-000000000721', 'claude-opus-4', 'running', now())",
        )
        .execute(store.test_pool())
        .await
        .expect("seed runs");

        let runs = runs_for_session_inner(
            &store,
            "00000000-0000-0000-0000-000000000721",
        )
        .await
        .expect("session runs");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].status, "completed");
        assert_eq!(runs[1].status, "running");

        let error = runs_for_session_inner(&store, "not-a-uuid")
            .await
            .expect_err("a malformed id is a typed rejection");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "invalid_argument"
        );
    }
}