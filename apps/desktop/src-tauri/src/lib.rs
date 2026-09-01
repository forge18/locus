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
    ids::{ArtifactId, BotId, PlanningWorkspaceId, ProjectId, RoutineId, RunId, TaskId},
    lsp::{DescriptorPin, LspDiagnostic},
    plugin::{builtin_manifests, PluginKind, PluginProcess, WorkItemProviderDescriptor},
    repo::{GitState, RepoManager},
    services::{
        artifact::{ArtifactComment, ArtifactContent, ArtifactKind, ArtifactRow},
        board::{BoardActor, BoardCommentOrigin, BoardEvidenceLink},
        bots::{
            Bot, BotContainerState, BotRoutine, RoutineAttribution, RoutineExecution,
            RoutineExecutionStatus,
        },
        capabilities::CapabilityPolicies,
        interact::InteractState,
        lint::discover as discover_linters,
        manage::TaskColumn,
        task::TaskDetailSummary,
        telemetry::{now_timestamp, CapturedEvent, Event, EventVerb},
    },
    store::{
        agents::ActivityCountsRow, qa::QaFindingRow, work_items::PersistedExternalCompletionStatus,
        Store,
    },
    work_item::{
        pull_from_plugin, push_note_to_plugin, push_status_to_plugin, snapshot_from_plugin,
        sync_capability_from_plugin, CompletionDelivery, ExternalWorkItemProvider,
        ImportedWorkItem, PluginWorkItemProvider, WorkItemError, WorkItemIdentity, WorkItemLookup,
        WorkItemPreview, WorkItemProviderConfig, WorkItemProviderId, WorkItemRegistry,
        WorkItemSnapshot, WorkItemSyncState,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[tauri::command]
async fn store_health(
    core: State<'_, Arc<Core>>,
) -> Result<locus_core::core::StoreHealth, IpcError> {
    if core.store().is_none() {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let _ = core.connect(&database_url).await;
        }
    }
    Ok(core.store_health())
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InteractChangedFileResponse {
    path: String,
    marker: String,
    additions: u32,
    removals: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InteractSessionResponse {
    id: String,
    project_id: String,
    project: String,
    name: String,
    agent: String,
    harness: String,
    branch: String,
    status: String,
    state: InteractState,
    board_task_id: Option<String>,
    run_id: Option<String>,
    run_status: Option<String>,
    model: Option<String>,
    permission_posture: String,
    created_at: Option<String>,
    repo: Option<String>,
    base_commit: Option<String>,
    changed_files: Vec<InteractChangedFileResponse>,
    cost: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InteractMutationResponse {
    session: InteractSessionResponse,
    branch: Option<String>,
}

fn interact_session_response(
    row: locus_core::store::interact::InteractSessionRow,
) -> InteractSessionResponse {
    InteractSessionResponse {
        id: row.id.to_string(),
        project_id: row.project_id.to_string(),
        project: row.project,
        name: row.name,
        agent: row.agent,
        harness: row.harness,
        branch: row.branch,
        status: row.status,
        state: row.state,
        board_task_id: row.board_task_id.map(|id| id.to_string()),
        run_id: row.run_id.map(|id| id.to_string()),
        run_status: row.run_status,
        model: row.model,
        permission_posture: row.permission_posture,
        created_at: row.created_at,
        repo: row.repo,
        base_commit: None,
        changed_files: Vec::new(),
        cost: Some("unknown".into()),
    }
}

fn interact_session_response_with_changes(
    row: locus_core::store::interact::InteractSessionRow,
) -> Result<InteractSessionResponse, String> {
    let remote = row.workspace_remote.clone();
    let branch = row.branch.clone();
    let mut response = interact_session_response(row);
    if let Some(remote) = remote {
        let repo = RepoManager::default();
        response.base_commit = repo.primary_commit_at_remote(&remote).ok();
        response.changed_files = repo
            .branch_changes_at_remote(&remote, &branch)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|change| InteractChangedFileResponse {
                path: change.path,
                marker: "M".into(),
                additions: change.additions,
                removals: change.removals,
            })
            .collect();
    }
    Ok(response)
}

fn parse_interact_session_id(value: &str) -> Result<locus_core::ids::SessionId, IpcError> {
    value
        .parse()
        .map_err(|_| IpcError::invalid_argument("Interact session id must be a UUID"))
}

#[tauri::command]
async fn interact_sessions_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<InteractSessionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = match project_id.as_deref() {
        Some(identifier) => Some(resolve_project_id(store, identifier).await?),
        None => None,
    };
    let rows = store
        .interact_sessions(project_id)
        .await
        .map_err(IpcError::internal)?;
    rows.into_iter()
        .map(interact_session_response_with_changes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn interact_session_create(
    core: State<'_, Arc<Core>>,
    project_id: String,
    name: String,
    model: Option<String>,
    repo_id: Option<String>,
) -> Result<InteractSessionResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let model = model
        .filter(|model| !model.trim().is_empty())
        .or_else(|| std::env::var("LOCUS_DEFAULT_MODEL_ID").ok())
        .unwrap_or_else(|| "unconfigured-model".into());
    let repo_id = repo_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(|_| IpcError::invalid_argument("repository id must be a UUID"))
        })
        .transpose()?;
    let session_id = store
        .create_interact_session(project_id, repo_id, &name, &model)
        .await
        .map_err(IpcError::internal)?;
    store
        .interact_session_for_project(project_id, session_id)
        .await
        .map_err(IpcError::internal)?
        .map(interact_session_response_with_changes)
        .transpose()
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::internal("created Interact session disappeared"))
}

#[tauri::command]
async fn interact_session_promote(
    core: State<'_, Arc<Core>>,
    project_id: String,
    session_id: String,
    task_id: Option<String>,
) -> Result<InteractSessionResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let session_id = parse_interact_session_id(&session_id)?;
    let task_id = task_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(|_| IpcError::invalid_argument("board task id must be a UUID"))
        })
        .transpose()?;
    store
        .promote_interact_session(project_id, session_id, task_id)
        .await
        .map_err(IpcError::internal)?;
    store
        .interact_session_for_project(project_id, session_id)
        .await
        .map_err(IpcError::internal)?
        .map(interact_session_response_with_changes)
        .transpose()
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("Interact session was not found"))
}

#[tauri::command]
async fn interact_session_discard(
    core: State<'_, Arc<Core>>,
    project_id: String,
    session_id: String,
) -> Result<InteractMutationResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let session_id = parse_interact_session_id(&session_id)?;
    let target = store
        .discard_interact_session(project_id, session_id)
        .await
        .map_err(IpcError::internal)?;
    let mut cleanup_errors = Vec::new();
    if let Some(container_id) = target.container_id.as_deref() {
        let mut runtime = core
            .connect_container_runtime()
            .map_err(IpcError::internal)?;
        if let Err(error) = runtime.stop_container(container_id) {
            cleanup_errors.push(format!("stop container: {error}"));
        }
        if let Err(error) = runtime.remove_container(container_id) {
            cleanup_errors.push(format!("remove container: {error}"));
        }
    }
    if let Some(remote) = target.workspace_remote.as_deref() {
        if let Err(error) = RepoManager::default().delete_interact_branch(remote, &target.branch) {
            cleanup_errors.push(format!("delete branch: {error}"));
        }
    }
    if !cleanup_errors.is_empty() {
        return Err(IpcError::internal(format!(
            "discard cleanup failed: {}",
            cleanup_errors.join("; ")
        )));
    }
    let session = store
        .interact_session_for_project(project_id, session_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("discarded Interact session was not found"))?;
    Ok(InteractMutationResponse {
        session: interact_session_response_with_changes(session).map_err(IpcError::internal)?,
        branch: Some(target.branch),
    })
}

#[tauri::command]
async fn interact_session_commit(
    core: State<'_, Arc<Core>>,
    project_id: String,
    session_id: String,
) -> Result<InteractMutationResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let session_id = parse_interact_session_id(&session_id)?;
    let session = store
        .commit_interact_session(project_id, session_id)
        .await
        .map_err(IpcError::internal)?;
    let container_id = session
        .container_id
        .as_deref()
        .ok_or_else(|| IpcError::invalid_argument("the Interact workspace is not running"))?;
    let branch = session.branch.clone();
    let command = vec![
        "git".into(),
        "push".into(),
        "locus".into(),
        format!("HEAD:refs/heads/{branch}"),
    ];
    let mut runtime = core
        .connect_container_runtime()
        .map_err(IpcError::internal)?;
    let result = runtime
        .exec(container_id, &command)
        .map_err(IpcError::internal)?;
    if result.status_code != 0 {
        return Err(IpcError::internal(format!(
            "push Interact branch failed: {}",
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    Ok(InteractMutationResponse {
        session: interact_session_response_with_changes(session).map_err(IpcError::internal)?,
        branch: Some(branch),
    })
}

#[tauri::command]
async fn interact_session_prompt(
    core: State<'_, Arc<Core>>,
    project_id: String,
    session_id: String,
    prompt: String,
) -> Result<(), IpcError> {
    if prompt.trim().is_empty() {
        return Err(IpcError::invalid_argument("prompt must not be empty"));
    }
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let session_id = parse_interact_session_id(&session_id)?;
    let run_id = store
        .active_interact_run(project_id, session_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::invalid_argument("the Interact session is not running"))?;
    core.prompt_run(run_id, prompt)
        .await
        .map_err(IpcError::internal)
}

/// Dispatch schedules and their execution history (slice 7)."}]} hab=functions.edit  时时彩?jsonikwembu? 天天众ьақә? Wait tool call malformed? Let's inspect result.}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduleResponse {
    id: String,
    project_id: String,
    project: String,
    name: String,
    cron: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduleExecutionResponse {
    id: String,
    schedule_name: String,
    project: String,
    status: String,
    scheduled_for: Option<String>,
    started_at: Option<String>,
    ended_at: Option<String>,
}

async fn schedules_list_inner(store: &Store) -> Result<Vec<ScheduleResponse>, IpcError> {
    store
        .schedules_list()
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| ScheduleResponse {
                    id: row.id.to_string(),
                    project_id: row.project_id.to_string(),
                    project: row.project,
                    name: row.name,
                    cron: row.cron_expression,
                    enabled: row.enabled,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn schedule_executions_inner(
    store: &Store,
    project_id: Option<&str>,
    limit: i64,
) -> Result<Vec<ScheduleExecutionResponse>, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    let limit = i64::try_from(limit.clamp(0, 500) as u64).unwrap_or(50);
    store
        .schedule_executions(scoped, limit)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| ScheduleExecutionResponse {
                    id: row.id.to_string(),
                    schedule_name: row.schedule_name,
                    project: row.project,
                    status: row.status,
                    scheduled_for: row.scheduled_for,
                    started_at: row.started_at,
                    ended_at: row.ended_at,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn dispatch_schedules(core: State<'_, Arc<Core>>) -> Result<Vec<ScheduleResponse>, IpcError> {
    let store = connected_store(&core).await?;
    schedules_list_inner(store).await
}

/// One session by id (the detail read) and the autorun switchboard.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutorunStateResponse {
    project_id: String,
    project: String,
    state: String,
}

async fn autorun_states_inner(store: &Store) -> Result<Vec<AutorunStateResponse>, IpcError> {
    store
        .autorun_states()
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| AutorunStateResponse {
                    project_id: row.project_id.to_string(),
                    project: row.project,
                    state: row.state,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn set_project_autorun_state_inner(
    store: &Store,
    project_id: &str,
    state: &str,
) -> Result<(), IpcError> {
    let pid = resolve_setup_project(store, project_id).await?;
    let state = match state {
        "on" => locus_core::runtime::dispatch::AutorunState::On,
        "off" => locus_core::runtime::dispatch::AutorunState::Off,
        "suspended" => locus_core::runtime::dispatch::AutorunState::Suspended,
        other => {
            return Err(IpcError::invalid_argument(format!(
                "unknown autorun state {other}"
            )))
        }
    };
    store
        .set_project_autorun_state(pid, state)
        .await
        .map_err(IpcError::internal)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GuardrailControlResponse {
    kind: String,
    value: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GuardrailSettingResponse {
    id: String,
    label: String,
    description: String,
    control: GuardrailControlResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GuardrailSectionResponse {
    id: String,
    label: String,
    settings: Vec<GuardrailSettingResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GuardrailSettingsRequest {
    max_iterations: i32,
    token_budget: Option<i64>,
    stuck_iterations: i32,
    kill_and_reassign: bool,
    global_parallelism: i32,
    per_project_parallelism: i32,
    priority_method: String,
    tie_break: String,
    change_lines_ceiling: Option<i32>,
    change_files_ceiling: Option<i32>,
    network_tier: String,
    block_system_changes: bool,
    autopilot: bool,
}

fn guardrail_stepper(id: &str, label: &str, value: impl Into<String>) -> GuardrailSettingResponse {
    GuardrailSettingResponse {
        id: id.into(),
        label: label.into(),
        description: "Installation default; applies to runs started after saving.".into(),
        control: GuardrailControlResponse {
            kind: "stepper".into(),
            value: serde_json::Value::String(value.into()),
        },
    }
}

fn guardrail_toggle(id: &str, label: &str, value: bool) -> GuardrailSettingResponse {
    GuardrailSettingResponse {
        id: id.into(),
        label: label.into(),
        description: "Installation default; applies to runs started after saving.".into(),
        control: GuardrailControlResponse {
            kind: "toggle".into(),
            value: serde_json::Value::Bool(value),
        },
    }
}

fn guardrail_select(id: &str, label: &str, value: &str) -> GuardrailSettingResponse {
    GuardrailSettingResponse {
        id: id.into(),
        label: label.into(),
        description: "Installation default; applies to runs started after saving.".into(),
        control: GuardrailControlResponse {
            kind: "select".into(),
            value: serde_json::Value::String(value.into()),
        },
    }
}

fn guardrail_settings_response(
    settings: &locus_core::store::guardrails::GuardrailSettings,
) -> Vec<GuardrailSectionResponse> {
    let defaults = &settings.defaults;
    let dispatch = &settings.dispatch;
    vec![
        GuardrailSectionResponse {
            id: "stopping".into(),
            label: "Stopping conditions".into(),
            settings: vec![
                guardrail_stepper(
                    "max-iterations",
                    "Max iterations",
                    defaults.max_iterations.to_string(),
                ),
                guardrail_stepper(
                    "token-budget",
                    "Token budget per run",
                    defaults.token_budget.map_or_else(
                        || "unlimited".into(),
                        |budget| format!("{}k", budget / 1000),
                    ),
                ),
                guardrail_stepper(
                    "stuck-detection",
                    "Stuck detection",
                    defaults.stuck_iterations.to_string(),
                ),
                guardrail_toggle(
                    "kill-reassign",
                    "Kill & reassign on stuck",
                    defaults.kill_and_reassign,
                ),
            ],
        },
        GuardrailSectionResponse {
            id: "parallelism".into(),
            label: "Parallelism".into(),
            settings: vec![
                guardrail_stepper(
                    "max-parallel-agents",
                    "Max parallel agents",
                    dispatch.global_parallelism.to_string(),
                ),
                guardrail_stepper(
                    "max-per-project",
                    "Max per project",
                    dispatch.per_project_parallelism.to_string(),
                ),
                guardrail_select(
                    "priority-method",
                    "Priority method",
                    &dispatch.priority_method.replace('_', " "),
                ),
                guardrail_select(
                    "tie-break",
                    "Tie-break",
                    &dispatch.tie_break.replace('_', " "),
                ),
            ],
        },
        GuardrailSectionResponse {
            id: "change-size".into(),
            label: "Change size".into(),
            settings: vec![
                guardrail_stepper(
                    "lines-changed",
                    "Lines changed ceiling",
                    defaults
                        .change_lines_ceiling
                        .map_or_else(|| "unlimited".into(), |value| value.to_string()),
                ),
                guardrail_stepper(
                    "files-touched",
                    "Files touched ceiling",
                    defaults
                        .change_files_ceiling
                        .map_or_else(|| "unlimited".into(), |value| value.to_string()),
                ),
            ],
        },
        GuardrailSectionResponse {
            id: "permissions".into(),
            label: "Permissions".into(),
            settings: vec![
                guardrail_select(
                    "network-tier",
                    "Network tier for new agents",
                    &defaults.network_tier,
                ),
                guardrail_toggle(
                    "block-system-changes",
                    "Block unapproved system changes",
                    defaults.block_system_changes,
                ),
                guardrail_toggle("autopilot", "Autopilot", defaults.autopilot),
            ],
        },
    ]
}

async fn guardrail_settings_inner(
    store: &Store,
) -> Result<Vec<GuardrailSectionResponse>, IpcError> {
    store
        .guardrail_settings()
        .await
        .map(|settings| guardrail_settings_response(&settings))
        .map_err(IpcError::internal)
}

async fn set_guardrail_settings_inner(
    store: &Store,
    request: GuardrailSettingsRequest,
) -> Result<Vec<GuardrailSectionResponse>, IpcError> {
    let settings = store
        .set_guardrail_settings(
            &locus_core::store::guardrails::GuardrailDefaultsRow {
                max_iterations: request.max_iterations,
                token_budget: request.token_budget,
                stuck_iterations: request.stuck_iterations,
                kill_and_reassign: request.kill_and_reassign,
                change_lines_ceiling: request.change_lines_ceiling,
                change_files_ceiling: request.change_files_ceiling,
                network_tier: request.network_tier,
                block_system_changes: request.block_system_changes,
                autopilot: request.autopilot,
            },
            &locus_core::store::guardrails::DispatchPolicyRow {
                global_parallelism: request.global_parallelism,
                per_project_parallelism: request.per_project_parallelism,
                priority_method: request.priority_method.replace(' ', "_"),
                tie_break: request.tie_break.replace(' ', "_"),
            },
        )
        .await
        .map_err(IpcError::internal)?;
    Ok(guardrail_settings_response(&settings))
}

#[tauri::command]
async fn settings_guardrails(
    core: State<'_, Arc<Core>>,
) -> Result<Vec<GuardrailSectionResponse>, IpcError> {
    guardrail_settings_inner(connected_store(&core).await?).await
}

#[tauri::command]
async fn settings_guardrails_set(
    core: State<'_, Arc<Core>>,
    request: GuardrailSettingsRequest,
) -> Result<Vec<GuardrailSectionResponse>, IpcError> {
    set_guardrail_settings_inner(connected_store(&core).await?, request).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanSummaryResponse {
    id: String,
    title: String,
    project: String,
    state: String,
    step: String,
    step_line: String,
    confidence: Option<f64>,
    open: Option<i32>,
    landed: Option<String>,
    age: String,
}

fn plan_step_label(stage: &str) -> Result<(&'static str, usize), IpcError> {
    let result = match stage {
        "inputs" => ("Inputs", 1),
        "orient" => ("Orient", 2),
        "converse" => ("Converse", 3),
        "synthesis" => ("Synthesis", 4),
        "recommend" => ("Recommend", 5),
        "decompose" => ("Decompose", 6),
        "approved" => ("Approved", 7),
        other => {
            return Err(IpcError::internal(format!(
                "plan has unknown stage `{other}`"
            )))
        }
    };
    Ok(result)
}

fn parse_plan_stage(
    stage: &str,
) -> Result<locus_core::services::planning::PlanningStage, IpcError> {
    match stage {
        "inputs" => Ok(locus_core::services::planning::PlanningStage::Inputs),
        "orient" => Ok(locus_core::services::planning::PlanningStage::Orient),
        "converse" => Ok(locus_core::services::planning::PlanningStage::Converse),
        "synthesis" => Ok(locus_core::services::planning::PlanningStage::Synthesis),
        "recommend" => Ok(locus_core::services::planning::PlanningStage::Recommend),
        "decompose" => Ok(locus_core::services::planning::PlanningStage::Decompose),
        "approved" => Ok(locus_core::services::planning::PlanningStage::Approved),
        other => Err(IpcError::invalid_argument(format!(
            "unknown plan stage `{other}`"
        ))),
    }
}

async fn plans_list_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<PlanSummaryResponse>, IpcError> {
    let project_id = match project_id {
        Some(project_id) => Some(resolve_setup_project(store, project_id).await?),
        None => None,
    };
    let rows = store
        .plans_list(project_id)
        .await
        .map_err(IpcError::internal)?;
    rows.into_iter()
        .map(|row| {
            let (step, step_number) = plan_step_label(&row.stage)?;
            Ok(PlanSummaryResponse {
                id: row.id.to_string(),
                title: row.title,
                project: row.project,
                state: row.state.clone(),
                step: step.into(),
                step_line: if row.state == "draft_rejected" {
                    format!(
                        "confidence {:.2} · open[{}]",
                        row.confidence.unwrap_or(0.0),
                        row.open_count
                    )
                } else {
                    format!("step {step_number} · {step}")
                },
                confidence: row.confidence,
                open: (row.state == "draft_rejected").then_some(row.open_count),
                landed: None,
                age: row.updated_at,
            })
        })
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanMutationResponse {
    updated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaletteSearchResponse {
    kind: String,
    project: String,
    label: String,
    locator: String,
    score: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceResponse {
    id: String,
    project_id: String,
    scope: String,
    lifecycle: String,
    current_revision: i32,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceApprovalResponse {
    workspace_id: String,
    revision: i32,
    task_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceRevisionResponse {
    id: String,
    workspace_id: String,
    revision: i32,
    state: Value,
    frozen_at: Option<String>,
    approved_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceSessionResponse {
    workspace_id: String,
    spec_id: Option<String>,
    session_id: String,
    linked_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceSpecResponse {
    id: String,
    workspace_id: String,
    repo_id: String,
    name: String,
    state: Value,
    stale: bool,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningWorkspaceTaskProvenanceResponse {
    id: String,
    materialization_id: String,
    workspace_id: String,
    revision_id: String,
    board_task_id: String,
    spec_id: String,
    requirement_id: Option<String>,
}

fn planning_workspace_response(
    row: locus_core::store::planning_workspace::PlanningWorkspaceRow,
) -> PlanningWorkspaceResponse {
    PlanningWorkspaceResponse {
        id: row.id.to_string(),
        project_id: row.project_id.to_string(),
        scope: row.scope,
        lifecycle: row.lifecycle,
        current_revision: row.current_revision,
        updated_at: row.updated_at,
    }
}

fn planning_workspace_revision_response(
    row: locus_core::store::planning_workspace::PlanningWorkspaceRevisionRow,
) -> PlanningWorkspaceRevisionResponse {
    PlanningWorkspaceRevisionResponse {
        id: row.id.to_string(),
        workspace_id: row.workspace_id.to_string(),
        revision: row.revision,
        state: row.state,
        frozen_at: row.frozen_at,
        approved_at: row.approved_at,
    }
}

fn planning_workspace_session_response(
    row: locus_core::store::planning_workspace::PlanningWorkspaceSessionRow,
) -> PlanningWorkspaceSessionResponse {
    PlanningWorkspaceSessionResponse {
        workspace_id: row.workspace_id.to_string(),
        spec_id: row.spec_id.map(|id| id.to_string()),
        session_id: row.session_id.to_string(),
        linked_at: row.linked_at,
    }
}

fn planning_workspace_spec_response(
    row: locus_core::store::planning_workspace::PlanningWorkspaceSpecRow,
) -> PlanningWorkspaceSpecResponse {
    PlanningWorkspaceSpecResponse {
        id: row.id.to_string(),
        workspace_id: row.workspace_id.to_string(),
        repo_id: row.repo_id.to_string(),
        name: row.name,
        state: row.state,
        stale: row.stale,
        updated_at: row.updated_at,
    }
}

fn planning_workspace_task_provenance_response(
    row: locus_core::store::planning_workspace::PlanningWorkspaceTaskProvenanceRow,
) -> PlanningWorkspaceTaskProvenanceResponse {
    PlanningWorkspaceTaskProvenanceResponse {
        id: row.id.to_string(),
        materialization_id: row.materialization_id.to_string(),
        workspace_id: row.workspace_id.to_string(),
        revision_id: row.revision_id.to_string(),
        board_task_id: row.board_task_id.to_string(),
        spec_id: row.spec_id.to_string(),
        requirement_id: row.requirement_id,
    }
}

fn parse_planning_workspace_id(value: &str) -> Result<PlanningWorkspaceId, IpcError> {
    value
        .parse()
        .map_err(|_| IpcError::invalid_argument("planning workspace id must be a UUID"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanRequirementRequest {
    id: String,
    body: String,
}

async fn plan_create_inner(
    store: &Store,
    project_id: &str,
    title: &str,
    goal: &str,
) -> Result<PlanSummaryResponse, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    let id = uuid::Uuid::new_v4();
    store
        .create_plan(id, project_id.into(), title, goal)
        .await
        .map_err(IpcError::internal)?;
    plans_list_inner(store, Some(&project_id.to_string()))
        .await?
        .into_iter()
        .find(|plan| plan.id == id.to_string())
        .ok_or_else(|| IpcError::internal("created plan disappeared"))
}

async fn plan_stage_set_inner(
    store: &Store,
    project_id: &str,
    plan_id: &str,
    stage: &str,
    description: &str,
    duration_seconds: Option<i64>,
) -> Result<(), IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    let plan_id = plan_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("plan id must be a UUID"))?;
    if !store
        .plan_belongs_to_project(plan_id, project_id)
        .await
        .map_err(IpcError::internal)?
    {
        return Err(IpcError::not_found(
            "plan was not found in the active project",
        ));
    }
    store
        .set_plan_stage(
            plan_id,
            parse_plan_stage(stage)?,
            description,
            duration_seconds,
        )
        .await
        .map_err(IpcError::internal)
}

async fn plan_requirements_set_inner(
    store: &Store,
    project_id: &str,
    plan_id: &str,
    requirements: Vec<PlanRequirementRequest>,
) -> Result<(), IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    let plan_id = plan_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("plan id must be a UUID"))?;
    if !store
        .plan_belongs_to_project(plan_id, project_id)
        .await
        .map_err(IpcError::internal)?
    {
        return Err(IpcError::not_found(
            "plan was not found in the active project",
        ));
    }
    let requirements = requirements
        .into_iter()
        .map(|requirement| {
            locus_core::services::planning::Requirement::new(requirement.id, requirement.body)
                .map_err(|error| IpcError::invalid_argument(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let spec = locus_core::services::planning::EditableSpec::new(requirements)
        .map_err(|error| IpcError::invalid_argument(error.to_string()))?;
    store
        .save_plan_requirements(plan_id, &spec)
        .await
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn search_all(
    core: State<'_, Arc<Core>>,
    query: String,
) -> Result<Vec<PaletteSearchResponse>, IpcError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    connected_store(&core)
        .await?
        .palette_search(&query)
        .await
        .map_err(IpcError::internal)
        .map(|rows| {
            rows.into_iter()
                .map(|row| PaletteSearchResponse {
                    kind: row.kind,
                    project: row.project,
                    label: row.label,
                    locator: row.locator,
                    score: row.score,
                })
                .collect()
        })
}

#[tauri::command]
async fn plans_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<PlanSummaryResponse>, IpcError> {
    let store = connected_store(&core).await?;
    plans_list_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn plan_create(
    core: State<'_, Arc<Core>>,
    project_id: String,
    title: String,
    goal: String,
) -> Result<PlanSummaryResponse, IpcError> {
    let store = connected_store(&core).await?;
    plan_create_inner(store, &project_id, &title, &goal).await
}

#[tauri::command]
async fn plan_stage_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    plan_id: String,
    stage: String,
    description: String,
    duration_seconds: Option<i64>,
) -> Result<PlanMutationResponse, IpcError> {
    let store = connected_store(&core).await?;
    plan_stage_set_inner(
        store,
        &project_id,
        &plan_id,
        &stage,
        &description,
        duration_seconds,
    )
    .await?;
    Ok(PlanMutationResponse { updated: true })
}

#[tauri::command]
async fn plan_requirements_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    plan_id: String,
    requirements: Vec<PlanRequirementRequest>,
) -> Result<PlanMutationResponse, IpcError> {
    let store = connected_store(&core).await?;
    plan_requirements_set_inner(store, &project_id, &plan_id, requirements).await?;
    Ok(PlanMutationResponse { updated: true })
}

#[tauri::command]
async fn planning_workspaces_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<PlanningWorkspaceResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = match project_id {
        Some(project_id) => Some(resolve_setup_project(store, &project_id).await?),
        None => None,
    };
    store
        .planning_workspaces(project_id)
        .await
        .map_err(IpcError::internal)
        .map(|rows| rows.into_iter().map(planning_workspace_response).collect())
}

#[tauri::command]
async fn planning_workspace_create(
    core: State<'_, Arc<Core>>,
    project_id: String,
    scope: String,
    brief: String,
) -> Result<PlanningWorkspaceResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let workspace_id = store
        .create_planning_workspace(project_id, &scope, &brief)
        .await
        .map_err(IpcError::internal)?;
    store
        .planning_workspace(project_id, workspace_id)
        .await
        .map_err(IpcError::internal)?
        .map(planning_workspace_response)
        .ok_or_else(|| IpcError::internal("created planning workspace disappeared"))
}

#[tauri::command]
async fn planning_workspace_revisions_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
) -> Result<Vec<PlanningWorkspaceRevisionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let workspace_id = parse_planning_workspace_id(&workspace_id)?;
    if store
        .planning_workspace(project_id, workspace_id)
        .await
        .map_err(IpcError::internal)?
        .is_none()
    {
        return Err(IpcError::not_found(
            "planning workspace was not found in the active project",
        ));
    }
    store
        .planning_workspace_revisions(project_id, workspace_id)
        .await
        .map_err(IpcError::internal)
        .map(|rows| {
            rows.into_iter()
                .map(planning_workspace_revision_response)
                .collect()
        })
}

#[tauri::command]
async fn planning_workspace_specs_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
) -> Result<Vec<PlanningWorkspaceSpecResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    store
        .planning_workspace_specs(project_id, parse_planning_workspace_id(&workspace_id)?)
        .await
        .map_err(IpcError::internal)
        .map(|rows| {
            rows.into_iter()
                .map(planning_workspace_spec_response)
                .collect()
        })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn planning_workspace_spec_save(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
    spec_id: Option<String>,
    repo_id: String,
    name: String,
    state: Value,
    stale: bool,
) -> Result<PlanningWorkspaceSpecResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let spec_id = spec_id
        .map(|value| {
            value.parse().map_err(|_| {
                IpcError::invalid_argument("planning workspace spec id must be a UUID")
            })
        })
        .transpose()?;
    let repo_id = repo_id.parse().map_err(|_| {
        IpcError::invalid_argument("planning workspace repository id must be a UUID")
    })?;
    let workspace_id = parse_planning_workspace_id(&workspace_id)?;
    let spec_id = store
        .save_planning_workspace_spec(
            project_id,
            workspace_id,
            spec_id,
            repo_id,
            &name,
            state,
            stale,
        )
        .await
        .map_err(IpcError::internal)?;
    store
        .planning_workspace_specs(project_id, workspace_id)
        .await
        .map_err(IpcError::internal)?
        .into_iter()
        .find(|spec| spec.id == spec_id)
        .map(planning_workspace_spec_response)
        .ok_or_else(|| IpcError::internal("saved planning workspace spec disappeared"))
}

#[tauri::command]
async fn planning_workspace_task_provenance_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
) -> Result<Vec<PlanningWorkspaceTaskProvenanceResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    store
        .planning_workspace_task_provenance(project_id, parse_planning_workspace_id(&workspace_id)?)
        .await
        .map_err(IpcError::internal)
        .map(|rows| {
            rows.into_iter()
                .map(planning_workspace_task_provenance_response)
                .collect()
        })
}

#[tauri::command]
async fn planning_workspace_decision_record(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
    affected_spec_ids: Vec<String>,
    decision: Value,
) -> Result<PlanMutationResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let spec_ids = affected_spec_ids
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| IpcError::invalid_argument("affected planning spec id must be a UUID"))
        })
        .collect::<Result<Vec<uuid::Uuid>, _>>()?;
    store
        .mark_planning_workspace_specs_stale(
            project_id,
            parse_planning_workspace_id(&workspace_id)?,
            &spec_ids,
            decision,
        )
        .await
        .map_err(IpcError::internal)?;
    Ok(PlanMutationResponse { updated: true })
}

#[tauri::command]
async fn planning_workspace_sessions_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
) -> Result<Vec<PlanningWorkspaceSessionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    store
        .planning_workspace_sessions(project_id, parse_planning_workspace_id(&workspace_id)?)
        .await
        .map_err(IpcError::internal)
        .map(|rows| {
            rows.into_iter()
                .map(planning_workspace_session_response)
                .collect()
        })
}

#[tauri::command]
async fn planning_workspace_session_link(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
    spec_id: Option<String>,
    session_id: String,
) -> Result<bool, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let spec_id = spec_id
        .map(|value| {
            value.parse().map_err(|_| {
                IpcError::invalid_argument("planning workspace spec id must be a UUID")
            })
        })
        .transpose()?;
    let session_id = session_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("planning session id must be a UUID"))?;
    store
        .link_planning_workspace_session(
            project_id,
            parse_planning_workspace_id(&workspace_id)?,
            spec_id,
            session_id,
        )
        .await
        .map_err(IpcError::internal)?;
    Ok(true)
}

#[tauri::command]
async fn planning_workspace_checkpoint_save(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
    expected_revision: i32,
    lifecycle: String,
    state: Value,
) -> Result<PlanningWorkspaceResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let workspace_id = parse_planning_workspace_id(&workspace_id)?;
    store
        .save_planning_workspace_checkpoint(
            project_id,
            workspace_id,
            expected_revision,
            &lifecycle,
            state,
        )
        .await
        .map_err(IpcError::internal)?;
    store
        .planning_workspace(project_id, workspace_id)
        .await
        .map_err(IpcError::internal)?
        .map(planning_workspace_response)
        .ok_or_else(|| IpcError::internal("planning workspace disappeared after checkpoint"))
}

#[tauri::command]
async fn planning_workspace_approve(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
    expected_revision: i32,
) -> Result<PlanningWorkspaceApprovalResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let workspace_id = parse_planning_workspace_id(&workspace_id)?;
    let task_ids = store
        .approve_planning_workspace(project_id, workspace_id, expected_revision)
        .await
        .map_err(IpcError::internal)?;
    Ok(PlanningWorkspaceApprovalResponse {
        workspace_id: workspace_id.to_string(),
        revision: expected_revision,
        task_ids: task_ids.into_iter().map(|id| id.to_string()).collect(),
    })
}

#[tauri::command]
async fn planning_workspace_delete(
    core: State<'_, Arc<Core>>,
    project_id: String,
    workspace_id: String,
) -> Result<bool, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    store
        .delete_planning_workspace(project_id, parse_planning_workspace_id(&workspace_id)?)
        .await
        .map_err(IpcError::internal)?;
    Ok(true)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BoardTaskResponse {
    id: String,
    project_id: String,
    repo_id: String,
    title: String,
    column: String,
    status: String,
    verify_command: String,
    assignee: Option<String>,
    gate: String,
    stuck_iterations: Option<u32>,
    max_iterations: u32,
    tools: String,
    tokens: Option<String>,
    workflow_id: Option<String>,
    root_session_id: Option<String>,
    child_run_ids: Vec<String>,
    evidence_ids: Vec<String>,
    external_link: Option<String>,
}

fn board_column_name(column: &str) -> Result<&'static str, IpcError> {
    match column {
        "ready" => Ok("ready"),
        "in_progress" => Ok("in_progress"),
        "testing" => Ok("testing"),
        "reviewing" => Ok("reviewing"),
        "waiting_for_approval" => Ok("waiting_for_approval"),
        "done" => Ok("done"),
        other => Err(IpcError::internal(format!(
            "board task has unknown column `{other}`"
        ))),
    }
}

fn board_task_response(
    row: locus_core::store::board::BoardTaskRow,
) -> Result<BoardTaskResponse, IpcError> {
    let column = board_column_name(&row.column_name)?;
    Ok(BoardTaskResponse {
        id: row.id.to_string(),
        project_id: row.project_id.to_string(),
        repo_id: row.repo_id.map_or_else(String::new, |id| id.to_string()),
        title: row.summary,
        column: column.into(),
        status: if row.blocked {
            "blocked".into()
        } else {
            "ok".into()
        },
        verify_command: row.verify_command.unwrap_or_default(),
        assignee: row.assigned_agent,
        gate: "unavailable".into(),
        stuck_iterations: None,
        max_iterations: 0,
        tools: "unavailable".into(),
        tokens: None,
        workflow_id: row.workflow_id.map(|id| id.to_string()),
        root_session_id: row.session_id.map(|id| id.to_string()),
        child_run_ids: row
            .child_run_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        evidence_ids: row
            .evidence_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        external_link: row.external_link,
    })
}

async fn board_tasks_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<BoardTaskResponse>, IpcError> {
    let project_id = match project_id {
        Some(project_id) => Some(resolve_setup_project(store, project_id).await?),
        None => None,
    };
    store
        .board_tasks(project_id)
        .await
        .map_err(IpcError::internal)?
        .into_iter()
        .map(board_task_response)
        .collect()
}

async fn task_detail_inner(
    store: &Store,
    project_id: &str,
    task_id: &str,
) -> Result<BoardTaskResponse, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    let task_id = task_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("task id must be a UUID"))?;
    store
        .board_task(project_id, task_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("task was not found in the active project"))
        .and_then(board_task_response)
}

#[tauri::command]
async fn board_tasks(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<BoardTaskResponse>, IpcError> {
    let store = connected_store(&core).await?;
    board_tasks_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn task_detail(
    core: State<'_, Arc<Core>>,
    project_id: String,
    task_id: String,
) -> Result<BoardTaskResponse, IpcError> {
    let store = connected_store(&core).await?;
    task_detail_inner(store, &project_id, &task_id).await
}

async fn task_create_inner(
    store: &Store,
    project_id: &str,
    repo_id: Option<&str>,
    summary: &str,
    workflow_def_id: &str,
) -> Result<BoardTaskResponse, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    let repo_id = repo_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(|_| IpcError::invalid_argument("repo id must be a UUID"))
        })
        .transpose()?;
    let workflow_def_id = workflow_def_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("workflow definition id must be a UUID"))?;
    if !store
        .workflow_definition_belongs_to_project(workflow_def_id, project_id)
        .await
        .map_err(IpcError::internal)?
    {
        return Err(IpcError::not_found(
            "workflow definition was not found in the active project",
        ));
    }
    store
        .create_board_task(project_id, repo_id, summary, workflow_def_id)
        .await
        .map(board_task_response)
        .map_err(IpcError::internal)?
}

#[tauri::command]
async fn task_create(
    core: State<'_, Arc<Core>>,
    project_id: String,
    repo_id: Option<String>,
    summary: String,
    workflow_def_id: String,
) -> Result<BoardTaskResponse, IpcError> {
    let store = connected_store(&core).await?;
    task_create_inner(
        store,
        &project_id,
        repo_id.as_deref(),
        &summary,
        &workflow_def_id,
    )
    .await
}

#[tauri::command]
async fn session(
    core: State<'_, Arc<Core>>,
    session_id: String,
) -> Result<SessionResponse, IpcError> {
    let store = connected_store(&core).await?;
    let session_id: uuid::Uuid = session_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("session id must be a UUID"))?;
    store
        .session(session_id)
        .await
        .map_err(IpcError::internal)?
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
        .ok_or_else(|| IpcError::not_found("session was not found"))
}

#[tauri::command]
async fn autorun_states(core: State<'_, Arc<Core>>) -> Result<Vec<AutorunStateResponse>, IpcError> {
    autorun_states_inner(connected_store(&core).await?).await
}

#[tauri::command]
async fn set_project_autorun_state(
    core: State<'_, Arc<Core>>,
    project_id: String,
    state: String,
) -> Result<(), IpcError> {
    set_project_autorun_state_inner(connected_store(&core).await?, &project_id, &state).await
}

#[tauri::command]
async fn dispatch_schedule_executions(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ScheduleExecutionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    schedule_executions_inner(store, project_id.as_deref(), limit.unwrap_or(50)).await
}

/// The Inbox (slice 7): pending human deliveries are the items; resolving one
/// drains the delivery and records the decision as a human reply on the thread.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InboxDeliveryResponse {
    id: String,
    thread_id: String,
    subject: String,
    body: String,
    sender_kind: String,
    project: String,
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolvedDeliveryResponse {
    id: String,
    subject: String,
    body: String,
    project: String,
    resolved_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InboxThroughputResponse {
    pending: usize,
    resolved_today: usize,
}

async fn inbox_list_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<InboxDeliveryResponse>, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    store
        .pending_human_inbox(scoped)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| InboxDeliveryResponse {
                    id: row.id.to_string(),
                    thread_id: row.thread_id.to_string(),
                    subject: row.subject,
                    body: row.body,
                    sender_kind: row.sender_kind,
                    project: row.project,
                    created_at: row.created_at,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn inbox_resolved_today_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<ResolvedDeliveryResponse>, IpcError> {
    let scoped = scope_project(store, project_id).await?;
    store
        .resolved_today(scoped)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| ResolvedDeliveryResponse {
                    id: row.id.to_string(),
                    subject: row.subject,
                    body: row.body,
                    project: row.project,
                    resolved_at: row.resolved_at,
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn inbox_throughput_inner(store: &Store) -> Result<InboxThroughputResponse, IpcError> {
    let pending = inbox_pending_count_inner(store).await?;
    let drained = store
        .resolved_today_count()
        .await
        .map_err(IpcError::internal)?;
    Ok(InboxThroughputResponse {
        pending,
        resolved_today: usize::try_from(drained)
            .map_err(|_| IpcError::internal("inbox count exceeds usize"))?,
    })
}

async fn inbox_resolve_inner(
    store: &Store,
    delivery_id: &str,
    comment: &str,
) -> Result<(), IpcError> {
    let delivery: uuid::Uuid = delivery_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("delivery id must be a UUID"))?;
    // An unknown delivery is a typed not-found, checked before anything moves.
    let thread_id = store
        .mail_thread_of_delivery(delivery)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("delivery was not found"))?;
    // The decision is auditable: the human's comment lands on the thread as a
    // reply, and the delivery drains so it leaves every pending view.
    if !comment.trim().is_empty() {
        let message_id = uuid::Uuid::new_v4();
        store
            .append_mail_message(message_id, thread_id, "human", None, comment)
            .await
            .map_err(IpcError::internal)?;
    }
    store
        .set_mail_delivery_status(delivery, "drained")
        .await
        .map_err(IpcError::internal)?;
    Ok(())
}

#[tauri::command]
async fn inbox_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<InboxDeliveryResponse>, IpcError> {
    let store = connected_store(&core).await?;
    inbox_list_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn inbox_resolved_today(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<ResolvedDeliveryResponse>, IpcError> {
    let store = connected_store(&core).await?;
    inbox_resolved_today_inner(store, project_id.as_deref()).await
}

#[tauri::command]
async fn inbox_throughput(core: State<'_, Arc<Core>>) -> Result<InboxThroughputResponse, IpcError> {
    let store = connected_store(&core).await?;
    inbox_throughput_inner(store).await
}

#[tauri::command]
async fn inbox_resolve(
    core: State<'_, Arc<Core>>,
    delivery_id: String,
    comment: String,
) -> Result<(), IpcError> {
    let store = connected_store(&core).await?;
    inbox_resolve_inner(store, &delivery_id, &comment).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCapabilityPolicyResponse {
    revision: i32,
    policies: CapabilityPolicies,
}

#[tauri::command]
async fn project_capability_policy(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<ProjectCapabilityPolicyResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let (revision, policies) = store
        .project_capability_policies(project_id)
        .await
        .map_err(IpcError::internal)?;
    Ok(ProjectCapabilityPolicyResponse { revision, policies })
}

#[tauri::command]
async fn project_capability_policy_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    policies: CapabilityPolicies,
) -> Result<ProjectCapabilityPolicyResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let revision = store
        .save_project_capability_policies(project_id, policies.clone())
        .await
        .map_err(IpcError::internal)?;
    Ok(ProjectCapabilityPolicyResponse { revision, policies })
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

async fn workflow_definitions_inner(
    store: &Store,
    project_id: &str,
) -> Result<Vec<ExternalWorkItemWorkflowResponse>, IpcError> {
    let project_id = resolve_project_id(store, project_id).await?;
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
async fn workflow_definitions(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<ExternalWorkItemWorkflowResponse>, IpcError> {
    workflow_definitions_inner(connected_store(&core).await?, &project_id).await
}

#[tauri::command]
async fn external_work_item_workflows(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<ExternalWorkItemWorkflowResponse>, IpcError> {
    workflow_definitions_inner(connected_store(&core).await?, &project_id).await
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

#[tauri::command]
async fn artifacts_list(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<ArtifactResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = match project_id {
        Some(project_id) => Some(resolve_setup_project(store, &project_id).await?),
        None => None,
    };
    let artifacts = store
        .review_artifacts(project_id)
        .await
        .map_err(IpcError::internal)?;
    Ok(artifacts.iter().map(artifact_response).collect())
}

#[tauri::command]
async fn artifact_comments(
    core: State<'_, Arc<Core>>,
    project_id: String,
    artifact_id: String,
) -> Result<Vec<ArtifactCommentResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_setup_project(store, &project_id).await?;
    let artifact_id: ArtifactId = artifact_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("artifact id must be a UUID"))?;
    let comments = store
        .artifact_comments(project_id, artifact_id)
        .await
        .map_err(IpcError::internal)?;
    Ok(comments.iter().map(artifact_comment_response).collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsQueryRequest {
    scope: String,
    range: String,
}

fn analytics_since_epoch(range: &str) -> Result<Option<i64>, IpcError> {
    let seconds = match range {
        "24h" => Some(24 * 60 * 60),
        "7d" => Some(7 * 24 * 60 * 60),
        "30d" => Some(30 * 24 * 60 * 60),
        "90d" => Some(90 * 24 * 60 * 60),
        "all" => None,
        _ => return Err(IpcError::invalid_argument("analytics range is invalid")),
    };
    Ok(seconds.map(|value| locus_core::services::analytics::current_unix_seconds() - value))
}

async fn activity_counts_inner(
    store: &Store,
    scope: &str,
    range: &str,
) -> Result<ActivityCountsRow, IpcError> {
    let project_id = match scope {
        "all" => None,
        identifier => Some(resolve_setup_project(store, identifier).await?),
    };
    let since_epoch = analytics_since_epoch(range)?;
    store
        .activity_counts(project_id, since_epoch)
        .await
        .map_err(IpcError::internal)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsMetricResponse {
    id: String,
    label: String,
    value: String,
    note: String,
}

async fn analytics_at_a_glance_inner(
    store: &Store,
    query: AnalyticsQueryRequest,
) -> Result<Vec<AnalyticsMetricResponse>, IpcError> {
    let counts = activity_counts_inner(store, &query.scope, &query.range).await?;
    let note = format!("scope {} · {}", query.scope, query.range);
    Ok(vec![
        AnalyticsMetricResponse {
            id: "sessions".into(),
            label: "Sessions".into(),
            value: counts.sessions.to_string(),
            note: note.clone(),
        },
        AnalyticsMetricResponse {
            id: "runs".into(),
            label: "Runs".into(),
            value: counts.runs.to_string(),
            note: note.clone(),
        },
        AnalyticsMetricResponse {
            id: "events".into(),
            label: "Events".into(),
            value: counts.events.to_string(),
            note: note.clone(),
        },
        AnalyticsMetricResponse {
            id: "tool-errors".into(),
            label: "Tool errors".into(),
            value: counts.errors.to_string(),
            note,
        },
    ])
}

#[tauri::command]
async fn analytics_at_a_glance(
    core: State<'_, Arc<Core>>,
    query: AnalyticsQueryRequest,
) -> Result<Vec<AnalyticsMetricResponse>, IpcError> {
    analytics_at_a_glance_inner(connected_store(&core).await?, query).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TelemetryMetricResponse {
    label: String,
    value: String,
    unit: Option<String>,
    bad: bool,
}

#[tauri::command]
async fn telemetry_metrics(
    core: State<'_, Arc<Core>>,
    query: AnalyticsQueryRequest,
) -> Result<Vec<TelemetryMetricResponse>, IpcError> {
    let counts =
        activity_counts_inner(connected_store(&core).await?, &query.scope, &query.range).await?;
    Ok(vec![
        TelemetryMetricResponse {
            label: "Sessions".into(),
            value: counts.sessions.to_string(),
            unit: None,
            bad: false,
        },
        TelemetryMetricResponse {
            label: "Runs".into(),
            value: counts.runs.to_string(),
            unit: None,
            bad: false,
        },
        TelemetryMetricResponse {
            label: "Events".into(),
            value: counts.events.to_string(),
            unit: None,
            bad: false,
        },
        TelemetryMetricResponse {
            label: "Tool errors".into(),
            value: counts.errors.to_string(),
            unit: None,
            bad: counts.errors > 0,
        },
    ])
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QaFindingResponse {
    id: String,
    source_id: String,
    severity: String,
    title: String,
    project: String,
    location: String,
    explanation: String,
    sent_to_inbox: bool,
}

fn qa_finding_response(row: QaFindingRow) -> QaFindingResponse {
    QaFindingResponse {
        id: row.id.to_string(),
        source_id: row.source_id,
        severity: row.severity,
        title: row.title,
        project: row.project,
        location: row.location,
        explanation: row.explanation,
        sent_to_inbox: row.sent_to_inbox,
    }
}

async fn qa_snapshot_inner(
    store: &Store,
    project_id: &str,
) -> Result<Vec<QaFindingResponse>, IpcError> {
    let project_id = resolve_setup_project(store, project_id).await?;
    store
        .qa_findings(project_id)
        .await
        .map(|rows| rows.into_iter().map(qa_finding_response).collect())
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn qa_snapshot(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<QaFindingResponse>, IpcError> {
    qa_snapshot_inner(connected_store(&core).await?, &project_id).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryFactResponse {
    id: String,
    title: String,
    score: Option<f64>,
    confidence: String,
    recall: String,
}

async fn memory_facts_inner(
    store: &Store,
    project_id: Option<&str>,
) -> Result<Vec<MemoryFactResponse>, IpcError> {
    let project_id = match project_id {
        Some(project_id) => Some(resolve_setup_project(store, project_id).await?),
        None => None,
    };
    store
        .memory_facts(project_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| MemoryFactResponse {
                    id: row.id.to_string(),
                    title: row.subject,
                    score: row.score,
                    confidence: row.confidence_state,
                    recall: if row.recall_count == 0 {
                        "not recalled".into()
                    } else {
                        format!("recalled {}×", row.recall_count)
                    },
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn memory_facts(
    core: State<'_, Arc<Core>>,
    project_id: Option<String>,
) -> Result<Vec<MemoryFactResponse>, IpcError> {
    memory_facts_inner(connected_store(&core).await?, project_id.as_deref()).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemoryConfidenceRequest {
    project_id: String,
    fact_id: String,
    confidence: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryMutationResponse {
    updated: bool,
}

async fn memory_confidence_set_inner(
    store: &Store,
    request: MemoryConfidenceRequest,
) -> Result<MemoryMutationResponse, IpcError> {
    let project_id = resolve_setup_project(store, &request.project_id).await?;
    let fact_id = request
        .fact_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("memory fact id must be a UUID"))?;
    let confidence = match request.confidence.as_str() {
        "verified" => locus_core::services::memory::ConfidenceState::Verified,
        "asserted" => locus_core::services::memory::ConfidenceState::Asserted,
        "decaying" => locus_core::services::memory::ConfidenceState::Decaying,
        "contradicted" => locus_core::services::memory::ConfidenceState::Contradicted,
        _ => return Err(IpcError::invalid_argument("memory confidence is invalid")),
    };
    let updated = store
        .set_memory_confidence_for_project(project_id, fact_id, confidence)
        .await
        .map_err(IpcError::internal)?;
    if !updated {
        return Err(IpcError::not_found(
            "memory fact was not found in this project",
        ));
    }
    Ok(MemoryMutationResponse { updated })
}

#[tauri::command]
async fn memory_confidence_set(
    core: State<'_, Arc<Core>>,
    project_id: String,
    fact_id: String,
    confidence: String,
) -> Result<MemoryMutationResponse, IpcError> {
    memory_confidence_set_inner(
        connected_store(&core).await?,
        MemoryConfidenceRequest {
            project_id,
            fact_id,
            confidence,
        },
    )
    .await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BotResponse {
    id: String,
    project_id: String,
    name: String,
    agent_def_id: String,
    home_session_id: String,
    active_run_id: Option<String>,
    harness: Option<String>,
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
            active_run_id: None,
            harness: None,
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

async fn require_bot_project(
    store: &Store,
    bot_id: BotId,
    project_id: ProjectId,
) -> Result<(), IpcError> {
    let bot = store
        .bot(bot_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found("bot was not found in the active project"))?;
    if bot.project_id != project_id {
        return Err(IpcError::not_found(
            "bot was not found in the active project",
        ));
    }
    Ok(())
}

async fn require_routine_project(
    store: &Store,
    routine_id: RoutineId,
    project_id: ProjectId,
) -> Result<(), IpcError> {
    if !store
        .bot_routine_belongs_to_project(routine_id, project_id)
        .await
        .map_err(IpcError::internal)?
    {
        return Err(IpcError::not_found(
            "bot routine was not found in the active project",
        ));
    }
    Ok(())
}

#[tauri::command]
async fn bots_list(
    core: State<'_, Arc<Core>>,
    project_id: String,
) -> Result<Vec<BotResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let bots = store.bots(project_id).await.map_err(IpcError::internal)?;
    let mut responses = Vec::with_capacity(bots.len());
    for bot in bots {
        let bot_id = bot.id;
        let active_run_id = store
            .active_bot_run(bot_id)
            .await
            .map_err(IpcError::internal)?
            .map(|id| id.to_string());
        let mut response = BotResponse::from(bot);
        response.active_run_id = active_run_id;
        response.harness = store
            .bot_harness(bot_id)
            .await
            .map_err(IpcError::internal)?;
        responses.push(response);
    }
    Ok(responses)
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
    project_id: String,
    bot_id: String,
) -> Result<Vec<BotRoutineResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let bot_id = parse_bot_id(&bot_id)?;
    require_bot_project(store, bot_id, project_id).await?;
    store
        .bot_routines(bot_id)
        .await
        .map(|routines| routines.into_iter().map(BotRoutineResponse::from).collect())
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_executions(
    core: State<'_, Arc<Core>>,
    project_id: String,
    bot_id: String,
) -> Result<Vec<BotRoutineExecutionResponse>, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let bot_id = parse_bot_id(&bot_id)?;
    require_bot_project(store, bot_id, project_id).await?;
    store
        .bot_routine_executions(bot_id)
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
    project_id: String,
    routine_id: String,
    enabled: bool,
) -> Result<(), IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let routine_id = parse_routine_id(&routine_id)?;
    require_routine_project(store, routine_id, project_id).await?;
    store
        .set_bot_routine_enabled(routine_id, enabled)
        .await
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_update(
    core: State<'_, Arc<Core>>,
    project_id: String,
    routine_id: String,
    prompt: String,
    cron_expression: String,
) -> Result<BotRoutineResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let routine_id = parse_routine_id(&routine_id)?;
    require_routine_project(store, routine_id, project_id).await?;
    store
        .update_bot_routine(routine_id, &prompt, &cron_expression)
        .await
        .map(BotRoutineResponse::from)
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_delete(
    core: State<'_, Arc<Core>>,
    project_id: String,
    routine_id: String,
) -> Result<(), IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let routine_id = parse_routine_id(&routine_id)?;
    require_routine_project(store, routine_id, project_id).await?;
    store
        .delete_bot_routine(routine_id)
        .await
        .map_err(IpcError::internal)
}

#[tauri::command]
async fn bot_routine_test(
    core: State<'_, Arc<Core>>,
    project_id: String,
    routine_id: String,
) -> Result<BotRoutineExecutionResponse, IpcError> {
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let routine_id = parse_routine_id(&routine_id)?;
    require_routine_project(store, routine_id, project_id).await?;
    let model =
        std::env::var("LOCUS_DEFAULT_MODEL_ID").unwrap_or_else(|_| "unconfigured-model".into());
    let (execution_id, bot_id, run_id) = store
        .test_bot_routine(routine_id, &model)
        .await
        .map_err(IpcError::internal)?;
    let dispatch = store
        .dispatch_run(run_id)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::internal("test routine run disappeared"))?;
    let mut runtime = core
        .connect_container_runtime()
        .map_err(IpcError::internal)?;
    if let Err(error) = core
        .spawn_dispatch_run(store, dispatch, &mut *runtime)
        .await
    {
        let _ = store.finish_bot_run(bot_id, run_id, false, None).await;
        let _ = store
            .complete_bot_routine_execution(
                execution_id,
                locus_core::services::bots::RoutineResult::failed(error.to_string()),
                Some(run_id),
            )
            .await;
        return Err(IpcError::internal(error));
    }
    store
        .bot_routine_execution(execution_id)
        .await
        .map_err(IpcError::internal)?
        .map(BotRoutineExecutionResponse::from)
        .ok_or_else(|| IpcError::not_found("test routine execution was not found"))
}

#[tauri::command]
async fn bot_prompt(
    core: State<'_, Arc<Core>>,
    project_id: String,
    bot_id: String,
    prompt: String,
) -> Result<(), IpcError> {
    if prompt.trim().is_empty() {
        return Err(IpcError::invalid_argument("prompt must not be empty"));
    }
    let store = connected_store(&core).await?;
    let project_id = resolve_project_id(store, &project_id).await?;
    let bot_id = parse_bot_id(&bot_id)?;
    require_bot_project(store, bot_id, project_id).await?;
    let run_id = if let Some(run_id) = store
        .active_bot_run(bot_id)
        .await
        .map_err(IpcError::internal)?
    {
        run_id
    } else {
        let run_id = RunId::generate();
        let model =
            std::env::var("LOCUS_DEFAULT_MODEL_ID").unwrap_or_else(|_| "unconfigured-model".into());
        store
            .start_bot_run(bot_id, run_id, &model)
            .await
            .map_err(IpcError::internal)?;
        let dispatch = store
            .dispatch_run(run_id)
            .await
            .map_err(IpcError::internal)?
            .ok_or_else(|| IpcError::internal("started bot run disappeared"))?;
        let mut runtime = core
            .connect_container_runtime()
            .map_err(IpcError::internal)?;
        if let Err(error) = core
            .spawn_dispatch_run(store, dispatch, &mut *runtime)
            .await
        {
            let _ = store.finish_bot_run(bot_id, run_id, false, None).await;
            return Err(IpcError::internal(error));
        }
        run_id
    };
    core.prompt_run(run_id, prompt)
        .await
        .map_err(IpcError::internal)
}

async fn agent_defs_list_inner(store: &Store) -> Result<Vec<AgentDefSummary>, IpcError> {
    store
        .agent_definitions()
        .await
        .map(|definitions| {
            definitions
                .into_iter()
                .map(|definition| AgentDefSummary {
                    name: definition.name,
                    version: u32::try_from(definition.version).unwrap_or_default(),
                })
                .collect()
        })
        .map_err(IpcError::internal)
}

async fn agent_def_inner(store: &Store, name: &str) -> Result<AgentDefResponse, IpcError> {
    let definition = store
        .latest_agent_definition(name)
        .await
        .map_err(IpcError::internal)?
        .ok_or_else(|| IpcError::not_found(format!("agent definition `{name}` was not found")))?;
    Ok(AgentDefResponse {
        name: definition.name,
        version: u32::try_from(definition.version)
            .map_err(|_| IpcError::internal("agent definition version is invalid"))?,
        frontmatter: definition.frontmatter,
        body: definition.body,
        warnings: Vec::new(),
    })
}

#[tauri::command]
async fn agent_defs_list(core: State<'_, Arc<Core>>) -> Result<Vec<AgentDefSummary>, IpcError> {
    agent_defs_list_inner(connected_store(&core).await?).await
}

#[tauri::command]
async fn agent_def(core: State<'_, Arc<Core>>, name: String) -> Result<AgentDefResponse, IpcError> {
    agent_def_inner(connected_store(&core).await?, &name).await
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

/// Emits one source-neutral event for the real-window integration harness only.
/// Release builds expose the command name but reject it before touching the collector.
#[tauri::command]
fn desktop_integration_emit_event(
    core: State<'_, Arc<Core>>,
    run_id: String,
    text: String,
) -> Result<(), IpcError> {
    if !cfg!(all(debug_assertions, feature = "webdriver")) {
        return Err(IpcError::not_found(
            "desktop integration event emission is unavailable",
        ));
    }
    let run_id = run_id
        .parse()
        .map_err(|_| IpcError::invalid_argument("integration run id must be a UUID"))?;
    core.collector().capture(
        run_id,
        CapturedEvent {
            verb: EventVerb::Assistant,
            ts: now_timestamp(),
            text: Some(text),
            tool: None,
            args: None,
            usage: None,
            raw: serde_json::json!({"integration": true}),
        },
    );
    Ok(())
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

    let builder = tauri::Builder::default();
    #[cfg(feature = "webdriver")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .manage(core)
        .manage(Arc::new(LspDiagnosticsSubscriptions::default()))
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
            project_capability_policy,
            project_capability_policy_set,
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
            interact_sessions_list,
            interact_session_create,
            interact_session_promote,
            interact_session_discard,
            interact_session_commit,
            interact_session_prompt,
            plans_list,
            plan_create,
            plan_stage_set,
            plan_requirements_set,
            search_all,
            planning_workspaces_list,
            planning_workspace_create,
            planning_workspace_revisions_list,
            planning_workspace_specs_list,
            planning_workspace_spec_save,
            planning_workspace_task_provenance_list,
            planning_workspace_decision_record,
            planning_workspace_sessions_list,
            planning_workspace_session_link,
            planning_workspace_checkpoint_save,
            planning_workspace_approve,
            planning_workspace_delete,
            board_tasks,
            task_detail,
            task_create,
            settings_guardrails,
            settings_guardrails_set,
            workflow_definitions,
            dispatch_schedules,
            dispatch_schedule_executions,
            session,
            autorun_states,
            set_project_autorun_state,
            inbox_list,
            inbox_resolved_today,
            inbox_throughput,
            inbox_resolve,
            harness_tier_grid,
            telemetry_subscribe,
            desktop_integration_emit_event,
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
            memory_facts,
            memory_confidence_set,
            analytics_at_a_glance,
            telemetry_metrics,
            qa_snapshot,
            bots_list,
            bot_create,
            bot_routines,
            bot_routine_executions,
            bot_routine_set_enabled,
            bot_routine_update,
            bot_routine_delete,
            bot_routine_test,
            bot_prompt,
            dispatch_stop_all,
            store_health,
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

    struct RunningRunSeed<'a> {
        project_id: &'a str,
        project_name: &'a str,
        agent_def_id: &'a str,
        agent_name: &'a str,
        session_id: &'a str,
        run_id: &'a str,
        status: &'a str,
    }

    async fn seed_running_run(store: &Store, seed: RunningRunSeed<'_>) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, $2)")
            .bind(seed.project_id)
            .bind(seed.project_name)
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, $2, 1, '{}'::jsonb, 'test agent')",
        )
        .bind(seed.agent_def_id)
        .bind(seed.agent_name)
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'shell session', 'agent/shell')",
        )
        .bind(seed.session_id)
        .bind(seed.project_id)
        .bind(seed.agent_def_id)
        .execute(store.test_pool())
        .await
        .expect("seed session");
        sqlx::query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status, started_at)
             VALUES ($1::uuid, $2::uuid, 'test-model', $3, now())",
        )
        .bind(seed.run_id)
        .bind(seed.session_id)
        .bind(seed.status)
        .execute(store.test_pool())
        .await
        .expect("seed run");
    }

    #[tokio::test]
    async fn running_count_and_cards_agree() {
        let (store, _cleanup) = test_store().await;
        seed_running_run(
            &store,
            RunningRunSeed {
                project_id: "00000000-0000-0000-0000-000000000401",
                project_name: "tapestry",
                agent_def_id: "00000000-0000-0000-0000-000000000411",
                agent_name: "builder",
                session_id: "00000000-0000-0000-0000-000000000421",
                run_id: "00000000-0000-0000-0000-000000000431",
                status: "running",
            },
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
            RunningRunSeed {
                project_id: "00000000-0000-0000-0000-000000000401",
                project_name: "tapestry",
                agent_def_id: "00000000-0000-0000-0000-000000000411",
                agent_name: "builder",
                session_id: "00000000-0000-0000-0000-000000000421",
                run_id: "00000000-0000-0000-0000-000000000431",
                status: "completed",
            },
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

        let scoped =
            sessions_list_inner(&store, Some("00000000-0000-0000-0000-000000000701"), 0, 100)
                .await
                .expect("scoped sessions");
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].project, "tapestry");

        let error =
            sessions_list_inner(&store, Some("00000000-0000-0000-0000-0000000007ff"), 0, 100)
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

        let runs = runs_for_session_inner(&store, "00000000-0000-0000-0000-000000000721")
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
/// Inbox queries and mutations: pending human deliveries, today's resolved list,
/// the pill's counts, and the drain-on-resolve decision with its audit reply.
#[cfg(test)]
mod inbox_flow {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-inbox").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the inbox test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the inbox test store");
        (store, cleanup)
    }

    /// One thread with one agent message and two deliveries: a human-pending one
    /// (the Inbox item) and an agent-pending one (never the Inbox's business).
    async fn seed_pending_delivery(store: &Store) {
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000801', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000841', 'builder', 1, '{}'::jsonb, 'test agent')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000851', '00000000-0000-0000-0000-000000000801', '00000000-0000-0000-0000-000000000841', 'inbox session', 'agent/inbox')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed session");
        sqlx::query(
            "INSERT INTO mail.threads (id, project_id, subject)
             VALUES ('00000000-0000-0000-0000-000000000811', '00000000-0000-0000-0000-000000000801', 'merge gate')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed thread");
        sqlx::query(
            "INSERT INTO mail.messages (id, thread_id, sender_kind, body)
             VALUES ('00000000-0000-0000-0000-000000000821', '00000000-0000-0000-0000-000000000811', 'agent', 'approve the merge'),
                    ('00000000-0000-0000-0000-000000000822', '00000000-0000-0000-0000-000000000811', 'agent', 'background note')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed messages");
        sqlx::query(
            "INSERT INTO mail.deliveries (id, message_id, recipient_kind, recipient_session_id, status)
             VALUES ('00000000-0000-0000-0000-000000000831', '00000000-0000-0000-0000-000000000821', 'human', NULL, 'pending'),
                    ('00000000-0000-0000-0000-000000000832', '00000000-0000-0000-0000-000000000822', 'agent', '00000000-0000-0000-0000-000000000851', 'pending')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed deliveries");
    }

    #[tokio::test]
    async fn the_inbox_lists_only_human_pending_deliveries() {
        let (store, _cleanup) = test_store().await;
        seed_pending_delivery(&store).await;

        let items = inbox_list_inner(&store, None).await.expect("list inbox");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].subject, "merge gate");
        assert_eq!(items[0].body, "approve the merge");
        assert_eq!(items[0].project, "tapestry");

        let throughput = inbox_throughput_inner(&store).await.expect("throughput");
        assert_eq!(throughput.pending, 1);
        assert_eq!(throughput.resolved_today, 0);
    }

    #[tokio::test]
    async fn resolving_drains_the_delivery_and_records_the_comment() {
        let (store, _cleanup) = test_store().await;
        seed_pending_delivery(&store).await;
        let delivery = "00000000-0000-0000-0000-000000000831";

        inbox_resolve_inner(&store, delivery, "approved — tests pass")
            .await
            .expect("resolve delivery");

        // The delivery drained, so the list and the pill both drop it.
        assert!(inbox_list_inner(&store, None)
            .await
            .expect("list")
            .is_empty());
        let throughput = inbox_throughput_inner(&store).await.expect("throughput");
        assert_eq!(throughput.pending, 0);
        assert_eq!(throughput.resolved_today, 1);

        // The decision is auditable on the thread as a human reply.
        let replies: Vec<String> = sqlx::query_scalar(
            "SELECT body FROM mail.messages WHERE sender_kind = 'human' ORDER BY created_at",
        )
        .fetch_all(store.test_pool())
        .await
        .expect("read replies");
        assert_eq!(replies, ["approved — tests pass"]);
    }

    #[tokio::test]
    async fn resolving_without_a_comment_still_drains() {
        let (store, _cleanup) = test_store().await;
        seed_pending_delivery(&store).await;

        inbox_resolve_inner(&store, "00000000-0000-0000-0000-000000000831", "")
            .await
            .expect("resolve without comment");
        assert!(inbox_list_inner(&store, None)
            .await
            .expect("list")
            .is_empty());
    }

    #[tokio::test]
    async fn an_unknown_delivery_is_rejected_not_emptied() {
        let (store, _cleanup) = test_store().await;
        seed_pending_delivery(&store).await;

        let error = inbox_resolve_inner(&store, "00000000-0000-0000-0000-0000000008ff", "x")
            .await
            .expect_err("unknown delivery rejected");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );

        let malformed = inbox_resolve_inner(&store, "not-a-uuid", "x")
            .await
            .expect_err("malformed id rejected");
        assert_eq!(
            serde_json::to_value(malformed).expect("serialize IPC error")["kind"],
            "invalid_argument"
        );
    }

    #[tokio::test]
    async fn the_resolved_today_list_scopes_by_project() {
        let (store, _cleanup) = test_store().await;
        seed_pending_delivery(&store).await;
        inbox_resolve_inner(&store, "00000000-0000-0000-0000-000000000831", "done")
            .await
            .expect("resolve");

        let today = inbox_resolved_today_inner(&store, None)
            .await
            .expect("resolved today");
        assert_eq!(today.len(), 1);
        assert_eq!(today[0].subject, "merge gate");

        // An existing second project has nothing resolved: the scope excludes
        // another project's rows rather than failing.
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000802', 'loom-db')")
            .execute(store.test_pool())
            .await
            .expect("seed second project");
        let scoped =
            inbox_resolved_today_inner(&store, Some("00000000-0000-0000-0000-000000000802"))
                .await
                .expect("scoped resolved today");
        assert!(scoped.is_empty());

        // An unknown scope is still a typed rejection, not an empty list.
        let error =
            inbox_resolved_today_inner(&store, Some("00000000-0000-0000-0000-0000000008ff"))
                .await
                .expect_err("unknown scope rejected");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }
}

/// Schedule queries: the dispatch schedules read from the existing
/// `workflows.schedules` and `workflows.executions` tables.
#[cfg(test)]
mod schedule_queries {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-schedules").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the schedule test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the schedule test store");
        (store, cleanup)
    }

    #[tokio::test]
    async fn schedules_read_with_their_project_and_workflow_name() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000901', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO workflows.workflow_defs (id, project_id, name, version, graph, spec, verify_command)
             VALUES ('00000000-0000-0000-0000-000000000911', '00000000-0000-0000-0000-000000000901', 'nightly reconcile', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed workflow def");
        sqlx::query(
            "INSERT INTO workflows.schedules (id, workflow_def_id, cron_expression)
             VALUES ('00000000-0000-0000-0000-000000000921', '00000000-0000-0000-0000-000000000911', '0 2 * * *')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed schedule");

        let schedules = schedules_list_inner(&store).await.expect("list schedules");
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].project, "tapestry");
        assert_eq!(schedules[0].name, "nightly reconcile");
        assert_eq!(schedules[0].cron, "0 2 * * *");
        assert!(schedules[0].enabled);
    }

    #[tokio::test]
    async fn schedule_executions_read_with_their_schedule_name() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000901', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO workflows.workflow_defs (id, project_id, name, version, graph, spec, verify_command)
             VALUES ('00000000-0000-0000-0000-000000000911', '00000000-0000-0000-0000-000000000901', 'nightly reconcile', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed workflow def");
        sqlx::query(
            "INSERT INTO workflows.schedules (id, workflow_def_id, cron_expression)
             VALUES ('00000000-0000-0000-0000-000000000921', '00000000-0000-0000-0000-000000000911', '0 2 * * *')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed schedule");
        sqlx::query(
            "INSERT INTO workflows.executions (id, workflow_def_id, schedule_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000931', '00000000-0000-0000-0000-000000000911', '00000000-0000-0000-0000-000000000921', 'completed', now() - interval '4 minutes')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed execution");

        let executions = schedule_executions_inner(&store, None, 50)
            .await
            .expect("list schedule executions");
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].schedule_name, "nightly reconcile");
        assert_eq!(executions[0].status, "completed");
    }
}
/// The autorun switchboard and session detail: the last slice-7 gaps.
#[cfg(test)]
mod autorun_and_session {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-autorun").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the autorun test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the autorun test store");
        (store, cleanup)
    }

    #[tokio::test]
    async fn autorun_states_list_every_project_defaulting_to_off() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000901', 'tapestry'), ('00000000-0000-0000-0000-000000000902', 'amq')")
            .execute(store.test_pool())
            .await
            .expect("seed projects");
        sqlx::query(
            "INSERT INTO core.project_autorun (project_id, enabled, state)
             VALUES ('00000000-0000-0000-0000-000000000901', TRUE, 'on')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed autorun");

        let states = autorun_states_inner(&store)
            .await
            .expect("list autorun states");
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].project, "amq");
        assert_eq!(states[0].state, "off");
        assert_eq!(states[1].state, "on");
    }

    #[tokio::test]
    async fn set_project_autorun_state_persists_and_refuses_archived() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000901', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");

        set_project_autorun_state_inner(&store, "00000000-0000-0000-0000-000000000901", "on")
            .await
            .expect("turn autorun on");
        let state = store
            .project_autorun_state("00000000-0000-0000-0000-000000000901".parse().expect("id"))
            .await
            .expect("read state");
        assert_eq!(state, locus_core::runtime::dispatch::AutorunState::On);

        let error =
            set_project_autorun_state_inner(&store, "00000000-0000-0000-0000-0000000009ff", "on")
                .await
                .expect_err("unknown project rejected");
        assert_eq!(
            serde_json::to_value(error).expect("serialize IPC error")["kind"],
            "not_found"
        );
    }

    #[tokio::test]
    async fn session_reads_one_session_and_rejects_unknown() {
        let (store, _cleanup) = test_store().await;
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ('00000000-0000-0000-0000-000000000901', 'tapestry')")
            .execute(store.test_pool())
            .await
            .expect("seed project");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000911', 'builder', 1, '{}'::jsonb, 'test agent')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent def");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000921', '00000000-0000-0000-0000-000000000901', '00000000-0000-0000-0000-000000000911', 'session detail', 'agent/detail')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed session");

        let session = store
            .session("00000000-0000-0000-0000-000000000921".parse().expect("id"))
            .await
            .expect("read session")
            .expect("session found");
        assert_eq!(session.project, "tapestry");
        assert_eq!(session.name, "session detail");

        let missing = store
            .session("00000000-0000-0000-0000-0000000009ff".parse().expect("id"))
            .await
            .expect("read missing session");
        assert!(missing.is_none());
    }
}

#[cfg(test)]
mod configuration_commands {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-configuration").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the configuration test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the configuration test store");
        (store, cleanup)
    }

    #[tokio::test]
    async fn plans_list_is_project_scoped_and_maps_durable_fields() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000a01', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000a02', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO core.plans
                (id, project_id, title, goal, stage, state, confidence, open_count)
             VALUES
                ('00000000-0000-0000-0000-000000000a11',
                 '00000000-0000-0000-0000-000000000a01',
                 'Tapestry plan', 'Ship the tapestry plan', 'recommend',
                 'draft_rejected', 0.626, 2),
                ('00000000-0000-0000-0000-000000000a12',
                 '00000000-0000-0000-0000-000000000a02',
                 'Loom plan', 'Ship the loom plan', 'approved',
                 'approved', NULL, 0)",
        )
        .execute(store.test_pool())
        .await
        .expect("seed plans");

        let plans = plans_list_inner(&store, Some("tapestry"))
            .await
            .expect("list tapestry plans");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].id, "00000000-0000-0000-0000-000000000a11");
        assert_eq!(plans[0].project, "tapestry");
        assert_eq!(plans[0].step, "Recommend");
        assert_eq!(plans[0].step_line, "confidence 0.63 · open[2]");
        assert_eq!(plans[0].confidence, Some(0.626));
        assert_eq!(plans[0].open, Some(2));
        assert!(plans[0].landed.is_none());
        assert!(!plans[0].age.is_empty());

        let unknown = plans_list_inner(&store, Some("00000000-0000-0000-0000-000000000aff"))
            .await
            .expect_err("unknown project rejected");
        assert!(matches!(unknown.kind, IpcErrorKind::NotFound));
    }

    #[tokio::test]
    async fn workflow_definitions_are_project_scoped() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000aa1', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000aa2', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO workflows.workflow_defs
                (id, project_id, name, version, graph, spec, verify_command)
             VALUES ('00000000-0000-0000-0000-000000000ab1',
                     '00000000-0000-0000-0000-000000000aa1', 'build', 1,
                     '{}'::jsonb, '{}'::jsonb, 'cargo test'),
                    ('00000000-0000-0000-0000-000000000ab2',
                     '00000000-0000-0000-0000-000000000aa2', 'review', 1,
                     '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed workflow definitions");

        let definitions = workflow_definitions_inner(&store, "tapestry")
            .await
            .expect("list tapestry workflows");
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "build");
        let missing = workflow_definitions_inner(&store, "missing-project")
            .await
            .expect_err("unknown workflow project rejected");
        assert!(matches!(missing.kind, IpcErrorKind::InvalidArgument));
    }

    #[tokio::test]
    async fn agent_definitions_read_latest_versions_from_store() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000a71', 'builder', 1, '{\"task_class\":\"code\"}', 'v1'),
                    ('00000000-0000-0000-0000-000000000a72', 'builder', 2, '{\"task_class\":\"research\"}', 'v2'),
                    ('00000000-0000-0000-0000-000000000a73', 'reviewer', 1, '{}', 'review')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent definitions");

        let definitions = agent_defs_list_inner(&store)
            .await
            .expect("list latest definitions");
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].name, "builder");
        assert_eq!(definitions[0].version, 2);
        let builder = agent_def_inner(&store, "builder")
            .await
            .expect("read builder definition");
        assert_eq!(builder.version, 2);
        assert_eq!(builder.frontmatter["task_class"], "research");
        let missing = agent_def_inner(&store, "missing")
            .await
            .expect_err("unknown definition rejected");
        assert!(matches!(missing.kind, IpcErrorKind::NotFound));
    }

    #[tokio::test]
    async fn guardrails_read_and_save_durable_defaults() {
        let (store, _cleanup) = test_store().await;
        let defaults = guardrail_settings_inner(&store)
            .await
            .expect("read guardrail settings");
        let max_iterations = defaults[0].settings[0].control.value.clone();
        assert_eq!(max_iterations, serde_json::json!("8"));
        assert_eq!(
            defaults[1].settings[0].control.value,
            serde_json::json!("6")
        );

        let updated = set_guardrail_settings_inner(
            &store,
            GuardrailSettingsRequest {
                max_iterations: 10,
                token_budget: Some(120_000),
                stuck_iterations: 4,
                kill_and_reassign: false,
                global_parallelism: 7,
                per_project_parallelism: 2,
                priority_method: "manual".into(),
                tie_break: "longest_waiting".into(),
                change_lines_ceiling: Some(500),
                change_files_ceiling: Some(15),
                network_tier: "allowlist".into(),
                block_system_changes: true,
                autopilot: true,
            },
        )
        .await
        .expect("save guardrail settings");
        assert_eq!(
            updated[0].settings[0].control.value,
            serde_json::json!("10")
        );
        assert_eq!(
            updated[1].settings[2].control.value,
            serde_json::json!("manual")
        );
        assert_eq!(
            updated[3].settings[2].control.value,
            serde_json::json!(true)
        );
    }

    #[tokio::test]
    async fn board_tasks_and_detail_are_project_scoped() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000a31', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000a32', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO board.tasks (id, project_id, summary, verify_command)
             VALUES ('00000000-0000-0000-0000-000000000a41',
                     '00000000-0000-0000-0000-000000000a31',
                     'Tapestry task', 'cargo test')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed board task");

        let tasks = board_tasks_inner(&store, Some("tapestry"))
            .await
            .expect("list tapestry tasks");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Tapestry task");
        assert_eq!(tasks[0].column, "ready");
        assert_eq!(tasks[0].status, "ok");
        assert_eq!(tasks[0].verify_command, "cargo test");

        let detail = task_detail_inner(&store, "tapestry", "00000000-0000-0000-0000-000000000a41")
            .await
            .expect("read tapestry task detail");
        assert_eq!(detail.id, tasks[0].id);
        let foreign = task_detail_inner(&store, "loom-db", "00000000-0000-0000-0000-000000000a41")
            .await
            .expect_err("cross-project task detail rejected");
        assert!(matches!(foreign.kind, IpcErrorKind::NotFound));
    }

    #[tokio::test]
    async fn task_create_requires_owned_workflow_and_persists_ready_task() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000a51', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000a52', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO workflows.workflow_defs
                (id, project_id, name, version, graph, spec, verify_command)
             VALUES ('00000000-0000-0000-0000-000000000a61',
                     '00000000-0000-0000-0000-000000000a51', 'build', 1,
                     '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed workflow");

        let task = task_create_inner(
            &store,
            "tapestry",
            None,
            "Created task",
            "00000000-0000-0000-0000-000000000a61",
        )
        .await
        .expect("create task");
        assert_eq!(task.title, "Created task");
        assert_eq!(task.column, "ready");

        let foreign = task_create_inner(
            &store,
            "loom-db",
            None,
            "Foreign task",
            "00000000-0000-0000-0000-000000000a61",
        )
        .await
        .expect_err("cross-project workflow rejected");
        assert!(matches!(foreign.kind, IpcErrorKind::NotFound));
    }

    #[tokio::test]
    async fn plan_mutations_require_project_ownership_and_persist() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000a21', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000a22', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");

        let created = plan_create_inner(&store, "tapestry", "New plan", "Do the thing")
            .await
            .expect("create plan");
        assert_eq!(created.project, "tapestry");
        assert_eq!(created.step, "Inputs");

        let foreign = plan_stage_set_inner(
            &store,
            "loom-db",
            &created.id,
            "orient",
            "orient the project",
            None,
        )
        .await
        .expect_err("cross-project stage update rejected");
        assert!(matches!(foreign.kind, IpcErrorKind::NotFound));

        plan_stage_set_inner(
            &store,
            "tapestry",
            &created.id,
            "converse",
            "ask the open questions",
            None,
        )
        .await
        .expect("set owned plan stage");
        plan_requirements_set_inner(
            &store,
            "tapestry",
            &created.id,
            vec![
                PlanRequirementRequest {
                    id: "R-01".into(),
                    body: "The plan must persist its goal.".into(),
                },
                PlanRequirementRequest {
                    id: "R-02".into(),
                    body: "The plan must preserve stable ids.".into(),
                },
            ],
        )
        .await
        .expect("save owned plan requirements");
        let requirements = store
            .plan_requirements(created.id.parse().expect("plan id"))
            .await
            .expect("read plan requirements");
        assert_eq!(requirements.len(), 2);
        assert_eq!(requirements[0].requirement_id, "R-01");
        assert!(requirements.iter().all(|row| row.changed));
    }
}

#[cfg(test)]
mod analytics_memory_queries {
    use super::*;
    use locus_core::testkit::postgres::{
        start_postgres_named, test_backup_config, NoopMigrationBackup,
    };

    async fn test_store() -> (Store, locus_core::testkit::postgres::DockerCleanup) {
        let (container, cleanup) = start_postgres_named("locus-tauri-memory").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the memory test store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
                &NoopMigrationBackup,
                &test_backup_config(),
            )
            .await
            .expect("run migrations for the memory test store");
        (store, cleanup)
    }

    #[tokio::test]
    async fn memory_facts_are_project_scoped_and_map_confidence() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000b01', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000b02', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO memory.store
                (id, project_id, scope, path, subject, category, body, provenance,
                 embedding, embedding_model, confidence, importance, strength,
                 confidence_state, recall_count)
             VALUES
                ('00000000-0000-0000-0000-000000000b11',
                 '00000000-0000-0000-0000-000000000b01', 'project', 'store.rs',
                 'Tapestry fact', 'fact', 'The project fact.', '{}'::jsonb,
                 '[1.0]'::vector, 'test', 0.94, 1.0, 1.0, 'verified', 31),
                ('00000000-0000-0000-0000-000000000b12',
                 '00000000-0000-0000-0000-000000000b02', 'project', 'store.rs',
                 'Loom fact', 'fact', 'The foreign fact.', '{}'::jsonb,
                 '[1.0]'::vector, 'test', 0.22, 1.0, 1.0, 'contradicted', 0)",
        )
        .execute(store.test_pool())
        .await
        .expect("seed memory facts");

        let facts = memory_facts_inner(&store, Some("tapestry"))
            .await
            .expect("list project memory facts");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].title, "Tapestry fact");
        assert_eq!(facts[0].confidence, "verified");
        assert_eq!(facts[0].score, Some(0.94));
        assert_eq!(facts[0].recall, "recalled 31×");

        let foreign = memory_facts_inner(&store, Some("loom-db"))
            .await
            .expect("list the other project memory facts");
        assert_eq!(foreign.len(), 1);
        assert_eq!(foreign[0].score, None);
        assert_eq!(foreign[0].recall, "not recalled");

        let updated = memory_confidence_set_inner(
            &store,
            MemoryConfidenceRequest {
                project_id: "tapestry".into(),
                fact_id: "00000000-0000-0000-0000-000000000b11".into(),
                confidence: "asserted".into(),
            },
        )
        .await
        .expect("adjudicate the owned fact");
        assert!(updated.updated);
        let foreign_update = memory_confidence_set_inner(
            &store,
            MemoryConfidenceRequest {
                project_id: "loom-db".into(),
                fact_id: "00000000-0000-0000-0000-000000000b11".into(),
                confidence: "verified".into(),
            },
        )
        .await
        .expect_err("cross-project adjudication rejected");
        assert!(matches!(foreign_update.kind, IpcErrorKind::NotFound));

        let unknown = memory_facts_inner(&store, Some("missing"))
            .await
            .expect_err("unknown project rejected");
        assert!(matches!(unknown.kind, IpcErrorKind::NotFound));
    }

    #[tokio::test]
    async fn analytics_counts_are_scoped_to_the_requested_project() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000c01', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000c02', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ('00000000-0000-0000-0000-000000000c11', 'builder', 1, '{}', 'builder'),
                    ('00000000-0000-0000-0000-000000000c12', 'auditor', 1, '{}', 'auditor')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed agent definitions");
        sqlx::query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ('00000000-0000-0000-0000-000000000c21',
                     '00000000-0000-0000-0000-000000000c01',
                     '00000000-0000-0000-0000-000000000c11', 'builder', 'feat/tapestry'),
                    ('00000000-0000-0000-0000-000000000c22',
                     '00000000-0000-0000-0000-000000000c02',
                     '00000000-0000-0000-0000-000000000c12', 'auditor', 'feat/loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed sessions");
        sqlx::query(
            "INSERT INTO agents.runs
                (id, session_id, resolved_model_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000c31',
                     '00000000-0000-0000-0000-000000000c21', 'test-model', 'completed', now()),
                    ('00000000-0000-0000-0000-000000000c32',
                     '00000000-0000-0000-0000-000000000c22', 'test-model', 'completed', now())",
        )
        .execute(store.test_pool())
        .await
        .expect("seed runs");
        sqlx::query(
            "INSERT INTO agents.events (id, run_id, seq, ts, verb, payload, raw)
             VALUES ('00000000-0000-0000-0000-000000000c41',
                     '00000000-0000-0000-0000-000000000c31', 0, now(), 'tool_error', '{}', '{}')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed telemetry event");

        let metrics = analytics_at_a_glance_inner(
            &store,
            AnalyticsQueryRequest {
                scope: "tapestry".into(),
                range: "all".into(),
            },
        )
        .await
        .expect("project analytics");
        assert_eq!(metrics[0].value, "1");
        assert_eq!(metrics[1].value, "1");
        assert_eq!(metrics[2].value, "1");
        assert_eq!(metrics[3].value, "1");

        sqlx::query(
            "INSERT INTO agents.runs
                (id, session_id, resolved_model_id, status, started_at)
             VALUES ('00000000-0000-0000-0000-000000000c33',
                     '00000000-0000-0000-0000-000000000c21', 'test-model',
                     'completed', now() - interval '2 days')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed old run");
        sqlx::query(
            "INSERT INTO agents.events (id, run_id, seq, ts, verb, payload, raw)
             VALUES ('00000000-0000-0000-0000-000000000c42',
                     '00000000-0000-0000-0000-000000000c33', 0, now(), 'assistant', '{}', '{}')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed recent event on old run");
        let recent = activity_counts_inner(&store, "tapestry", "24h")
            .await
            .expect("count recent activity");
        assert_eq!(recent.sessions, 1);
        assert_eq!(recent.runs, 1);
        assert_eq!(recent.events, 2);
        assert_eq!(recent.errors, 1);

        let all_metrics = analytics_at_a_glance_inner(
            &store,
            AnalyticsQueryRequest {
                scope: "all".into(),
                range: "all".into(),
            },
        )
        .await
        .expect("global analytics");
        assert_eq!(all_metrics[0].value, "2");
        assert_eq!(all_metrics[1].value, "3");
        assert_eq!(all_metrics[2].value, "2");
        assert_eq!(all_metrics[3].value, "1");
    }

    #[tokio::test]
    async fn qa_snapshot_is_project_scoped() {
        let (store, _cleanup) = test_store().await;
        sqlx::query(
            "INSERT INTO core.projects (id, name)
             VALUES ('00000000-0000-0000-0000-000000000d01', 'tapestry'),
                    ('00000000-0000-0000-0000-000000000d02', 'loom-db')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed projects");
        sqlx::query(
            "INSERT INTO core.qa_check_runs
                (id, project_id, check_source_id, trigger, started_at, finished_at)
             VALUES ('00000000-0000-0000-0000-000000000d11',
                     '00000000-0000-0000-0000-000000000d01', 'unit-tests', 'manual', now(), now()),
                    ('00000000-0000-0000-0000-000000000d12',
                     '00000000-0000-0000-0000-000000000d02', 'unit-tests', 'manual', now(), now())",
        )
        .execute(store.test_pool())
        .await
        .expect("seed QA check runs");
        sqlx::query(
            "INSERT INTO core.qa_findings
                (id, check_run_id, project_id, check_source_id, severity, title, location, explanation)
             VALUES ('00000000-0000-0000-0000-000000000d21',
                     '00000000-0000-0000-0000-000000000d11',
                     '00000000-0000-0000-0000-000000000d01', 'unit-tests', 'fail',
                     'Tapestry failure', 'src/lib.rs:1', 'The test failed.')",
        )
        .execute(store.test_pool())
        .await
        .expect("seed QA finding");

        let findings = qa_snapshot_inner(&store, "tapestry")
            .await
            .expect("list project QA findings");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "Tapestry failure");
        assert_eq!(findings[0].project, "tapestry");
        assert!(!findings[0].sent_to_inbox);

        let foreign = qa_snapshot_inner(&store, "loom-db")
            .await
            .expect("list foreign QA findings");
        assert!(foreign.is_empty());
    }
}
