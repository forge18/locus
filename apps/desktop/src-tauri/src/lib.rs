use std::path::PathBuf;

use locus_core::{
    materialize::{reports_for_registry, MaterializationReport},
    registry::load_from_directory,
    telemetry::{Event, EventCollector},
};
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};

const MODEL_TIERS: [&str; 4] = ["low", "medium", "high", "xhigh"];
const HARNESS_REGISTRY: &str = "../../../harnesses";

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
pub struct HarnessTierGridHarness {
    pub name: String,
    pub models: Option<Vec<String>>,
    pub tiers: Vec<ModelTierSetting>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EventCollector::new(1_024))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            harness_tier_grid,
            telemetry_subscribe,
            materialization_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

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
