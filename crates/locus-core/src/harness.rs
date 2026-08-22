//! Project harness selection policy.
//!
//! A project can launch only a permitted harness through a registered adapter and a provider that
//! is both configured and declared compatible by that harness. Model routing remains separate.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::routing::RoutingDefaults;

/// The identity of a Locus-side adapter that can launch a harness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HarnessAdapter {
    pub identity: String,
}

/// The provider and adapter facts declared for one harness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HarnessDescriptor {
    pub identifier: String,
    pub adapter: Option<HarnessAdapter>,
    pub compatible_providers: BTreeSet<String>,
    pub defaults: RoutingDefaults,
}

/// The project-specific set of harnesses and providers available for selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectHarnessPolicy {
    pub permitted_harnesses: BTreeSet<String>,
    pub configured_providers: BTreeSet<String>,
}

/// A harness and provider that passed the project selection gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedHarness {
    pub harness: String,
    pub adapter: HarnessAdapter,
    pub provider: String,
    pub defaults: RoutingDefaults,
}

/// Why a project cannot select a requested harness/provider pair.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HarnessSelectionError {
    #[error("harness `{0}` is not permitted by this project")]
    HarnessNotPermitted(String),
    #[error("harness `{0}` has no registered adapter")]
    AdapterUnavailable(String),
    #[error("provider `{0}` is not configured")]
    ProviderNotConfigured(String),
    #[error("harness `{harness}` is not compatible with provider `{provider}`")]
    ProviderIncompatible { harness: String, provider: String },
}

impl ProjectHarnessPolicy {
    /// Select a project-permitted, adapter-backed harness for a configured compatible provider.
    pub fn select(
        &self,
        harness: &HarnessDescriptor,
        provider: &str,
    ) -> Result<SelectedHarness, HarnessSelectionError> {
        if !self.permitted_harnesses.contains(&harness.identifier) {
            return Err(HarnessSelectionError::HarnessNotPermitted(
                harness.identifier.clone(),
            ));
        }
        let Some(adapter) = harness.adapter.clone() else {
            return Err(HarnessSelectionError::AdapterUnavailable(
                harness.identifier.clone(),
            ));
        };
        if !self.configured_providers.contains(provider) {
            return Err(HarnessSelectionError::ProviderNotConfigured(
                provider.into(),
            ));
        }
        if !harness.compatible_providers.contains(provider) {
            return Err(HarnessSelectionError::ProviderIncompatible {
                harness: harness.identifier.clone(),
                provider: provider.into(),
            });
        }

        Ok(SelectedHarness {
            harness: harness.identifier.clone(),
            adapter,
            provider: provider.into(),
            defaults: harness.defaults.clone(),
        })
    }
}

#[cfg(test)]
#[test]
fn project_selection_gate() {
    let policy = ProjectHarnessPolicy {
        permitted_harnesses: ["claude", "aider"].into_iter().map(str::to_owned).collect(),
        configured_providers: ["anthropic", "openai"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    };
    let claude = HarnessDescriptor {
        identifier: "claude".into(),
        adapter: Some(HarnessAdapter {
            identity: "claude-acp-v3".into(),
        }),
        compatible_providers: ["anthropic", "openrouter"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        defaults: RoutingDefaults {
            model_id: "claude-sonnet-4-6".into(),
            effort: "medium".into(),
        },
    };
    let aider = HarnessDescriptor {
        identifier: "aider".into(),
        adapter: None,
        compatible_providers: ["anthropic"].into_iter().map(str::to_owned).collect(),
        defaults: RoutingDefaults {
            model_id: "claude-sonnet-4-6".into(),
            effort: "medium".into(),
        },
    };

    assert_eq!(
        policy.select(&claude, "anthropic"),
        Ok(SelectedHarness {
            harness: "claude".into(),
            adapter: HarnessAdapter {
                identity: "claude-acp-v3".into(),
            },
            provider: "anthropic".into(),
            defaults: RoutingDefaults {
                model_id: "claude-sonnet-4-6".into(),
                effort: "medium".into(),
            },
        })
    );
    assert_eq!(
        policy.select(&aider, "anthropic"),
        Err(HarnessSelectionError::AdapterUnavailable("aider".into()))
    );
    assert_eq!(
        policy.select(&claude, "openrouter"),
        Err(HarnessSelectionError::ProviderNotConfigured(
            "openrouter".into()
        ))
    );
    assert_eq!(
        policy.select(&claude, "openai"),
        Err(HarnessSelectionError::ProviderIncompatible {
            harness: "claude".into(),
            provider: "openai".into(),
        })
    );

    let unpermitted = HarnessDescriptor {
        identifier: "codex".into(),
        adapter: Some(HarnessAdapter {
            identity: "codex-acp-v3".into(),
        }),
        compatible_providers: ["openai"].into_iter().map(str::to_owned).collect(),
        defaults: RoutingDefaults {
            model_id: "gpt-5.2".into(),
            effort: "medium".into(),
        },
    };
    assert_eq!(
        policy.select(&unpermitted, "openai"),
        Err(HarnessSelectionError::HarnessNotPermitted("codex".into()))
    );
}
