use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use locus_core::{
    core::Core,
    harness::materialize::report::{reports_for_registry, MaterializationReport},
    ids::{ArtifactId, ProjectId, RunId},
    lsp::{DescriptorPin, LspDiagnostic},
    repo::GitState,
    services::{
        agents::{seeded_definitions, AgentDefinition},
        artifact::{ArtifactComment, ArtifactContent, ArtifactKind, ArtifactRow, ArtifactStore},
        lint::discover as discover_linters,
        telemetry::Event,
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
            app.set_menu(Menu::with_items(app, &[&command_palette])?)?;
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
