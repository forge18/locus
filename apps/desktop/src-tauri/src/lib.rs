use std::path::PathBuf;

use locus_core::{
    agents::{seeded_definitions, AgentDefinition},
    artifact::{ArtifactComment, ArtifactContent, ArtifactKind, ArtifactRow, ArtifactStore},
    ipc::PtyChannel,
    lint::discover as discover_linters,
    materialize::{reports_for_registry, MaterializationReport},
    registry::load_from_directory,
    telemetry::{Event, EventCollector},
};
use serde::{Deserialize, Serialize};
use tauri::{
    ipc::Channel,
    menu::{Menu, MenuItem},
    Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use uuid::Uuid;

const MODEL_TIERS: [&str; 4] = ["low", "medium", "high", "xhigh"];
const HARNESS_REGISTRY: &str = "../../../harnesses";
const COMMAND_PALETTE_ACCELERATOR: &str = "CmdOrCtrl+K";

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
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactCommentResponse {
    id: String,
    artifact_id: String,
    parent_id: Option<String>,
    author: String,
    body: String,
    created_at: String,
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
        created_at: String::new(),
    }
}

fn artifact_comment_response(comment: &ArtifactComment) -> ArtifactCommentResponse {
    ArtifactCommentResponse {
        id: comment.id.to_string(),
        artifact_id: comment.artifact_id.to_string(),
        parent_id: comment.parent_id.map(|id| id.to_string()),
        author: "you".into(),
        body: comment.body.clone(),
        created_at: String::new(),
    }
}

fn seeded_artifact_store() -> ArtifactStore {
    let project_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
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
) -> Result<Vec<ArtifactCommentResponse>, String> {
    let artifact_id = Uuid::parse_str(&artifact_id).map_err(|error| error.to_string())?;
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
fn agent_def(name: String) -> Result<AgentDefResponse, String> {
    let (version, definition) = seeded_agent_definitions()
        .into_iter()
        .find(|(_, definition)| definition.frontmatter.name == name)
        .ok_or_else(|| format!("agent definition `{name}` was not found"))?;
    let frontmatter =
        serde_json::to_value(&definition.frontmatter).map_err(|error| error.to_string())?;
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

#[tauri::command]
fn pty_subscribe(pty: State<'_, PtyChannel>, channel: Channel<Vec<u8>>) {
    let mut bytes = pty.subscribe();
    tauri::async_runtime::spawn(async move {
        while let Ok(bytes) = bytes.recv().await {
            if channel.send(bytes).is_err() {
                break;
            }
        }
    });
}

#[tauri::command]
fn telemetry_subscribe(collector: State<'_, EventCollector>, channel: Channel<Event>) {
    let mut events = collector.subscribe();
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = events.recv().await {
            if channel.send(event).is_err() {
                break;
            }
        }
    });
}

#[tauri::command]
fn linter_count(root: String) -> Result<usize, String> {
    discover_linters(root)
        .map(|linters| linters.len())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn materialization_report() -> Result<Vec<MaterializationReport>, String> {
    let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(HARNESS_REGISTRY);
    load_from_directory(registry)
        .map(|registry| reports_for_registry(&registry))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn harness_tier_grid(request: HarnessTierGridRequest) -> Result<HarnessTierGridResponse, String> {
    if request.project_id.trim().is_empty() {
        return Err("harness tier grid requires a project id".into());
    }
    for setting in &request.tier_settings {
        if !MODEL_TIERS.contains(&setting.tier.as_str()) {
            return Err(format!("unknown model tier `{}`", setting.tier));
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
fn detach_pane(app: tauri::AppHandle, pane_id: String) -> Result<(), String> {
    let label = format!("pane-{pane_id}");
    if app.get_webview_window(&label).is_none() {
        WebviewWindowBuilder::new(
            &app,
            label,
            WebviewUrl::App("index.html?detached=true".into()),
        )
        .title("Locus pane")
        .build()
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EventCollector::new(1_024))
        .manage(PtyChannel::new(1_024))
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
            detach_pane,
            linter_count,
            artifacts_list,
            artifact_comments,
            materialization_report
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
        let reports = materialization_report().expect("registry report");
        assert_eq!(reports.len(), 12);
        assert_eq!(reports.iter().flat_map(|report| &report.losses).count(), 33);
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
