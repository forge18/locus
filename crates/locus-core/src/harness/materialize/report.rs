//! What each harness loses relative to a native mechanism, computed from the registry.

use super::*;

/// A loss in native behavior that the Extensions screen must show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationLoss {
    pub extension: String,
    pub weaker_than_native: String,
}

/// The registry-derived report displayed by the Extensions screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationReport {
    pub harness: String,
    pub losses: Vec<MaterializationLoss>,
}

pub fn reports_for_registry(registry: &HarnessRegistry) -> Vec<MaterializationReport> {
    registry
        .iter()
        .map(|harness| MaterializationReport {
            harness: harness.name.clone(),
            losses: harness
                .layout
                .named_entries()
                .into_iter()
                .filter_map(|(extension, entry)| {
                    entry
                        .weaker_than_native
                        .as_ref()
                        .map(|loss| MaterializationLoss {
                            extension: extension.into(),
                            weaker_than_native: loss.clone(),
                        })
                })
                .collect(),
        })
        .collect()
}
