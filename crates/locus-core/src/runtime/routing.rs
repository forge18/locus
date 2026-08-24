//! Six-band autorouting policy and its durable run decision.

#[cfg(test)]
use crate::ids::{AgentDefId, ProjectId, RunId, SessionId};
use std::{collections::BTreeMap, fmt};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Ordered task-complexity bands used by autorouting.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComplexityBand {
    XtraLow,
    Low,
    Medium,
    High,
    XtraHigh,
    Max,
}

impl ComplexityBand {
    const ORDERED: [Self; 6] = [
        Self::XtraLow,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::XtraHigh,
        Self::Max,
    ];

    fn upward_from(self) -> impl Iterator<Item = Self> {
        Self::ORDERED.into_iter().filter(move |band| *band >= self)
    }
}

/// The one effort vocabulary shared by routing, Harnesses, and Plan → Decompose.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingEffort {
    Low,
    Medium,
    High,
    Xhigh,
}

impl RoutingEffort {
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Xhigh];
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" => Ok(Self::Xhigh),
            value => bail!("unsupported routing effort `{value}`"),
        }
    }
}

impl fmt::Display for RoutingEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        })
    }
}
impl From<&str> for RoutingEffort {
    fn from(value: &str) -> Self {
        Self::parse(value).unwrap_or_else(|error| panic!("{error}"))
    }
}
impl PartialEq<&str> for RoutingEffort {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

/// The model and effort selected whenever autorouting is disabled.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingDefaults {
    pub model_id: String,
    pub effort: RoutingEffort,
}

/// Settings for one complexity band. A missing `model_id` deliberately falls upward.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingBand {
    pub model_id: Option<String>,
    pub effort: RoutingEffort,
    pub approval_required: bool,
    pub when_to_use: String,
}

/// The settings-owned routing policy for one harness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutoroutingPolicy {
    pub enabled: bool,
    pub bands: BTreeMap<ComplexityBand, RoutingBand>,
}

/// The immutable selection that is stored against a run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingDecision {
    pub requested_band: ComplexityBand,
    pub selected_band: Option<ComplexityBand>,
    pub model_id: String,
    pub effort: RoutingEffort,
    pub approval_required: bool,
}

impl AutoroutingPolicy {
    /// Resolve a classified task without selecting a weaker configured band.
    pub fn route(
        &self,
        requested_band: ComplexityBand,
        defaults: &RoutingDefaults,
    ) -> Result<RoutingDecision> {
        if !self.enabled {
            require_nonempty("default model", &defaults.model_id)?;
            let _ = defaults.effort;

            return Ok(RoutingDecision {
                requested_band,
                selected_band: None,
                model_id: defaults.model_id.clone(),
                effort: defaults.effort,
                approval_required: false,
            });
        }

        for selected_band in requested_band.upward_from() {
            let Some(band) = self.bands.get(&selected_band) else {
                continue;
            };
            let Some(model_id) = &band.model_id else {
                continue;
            };
            require_nonempty("band model", model_id)?;
            let _ = band.effort;
            require_nonempty("band when-to-use prose", &band.when_to_use)?;
            return Ok(RoutingDecision {
                requested_band,
                selected_band: Some(selected_band),
                model_id: model_id.clone(),
                effort: band.effort,
                approval_required: band.approval_required,
            });
        }

        bail!("no configured model at or above `{requested_band:?}`")
    }
}

fn require_nonempty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{name} must not be empty")
    }
    Ok(())
}

#[cfg(test)]
use crate::store::backup::RetainedBackupConfig;
#[cfg(test)]
use sqlx::query_as;

#[cfg(test)]
use crate::store::Store;
#[cfg(test)]
use sqlx::query;
#[cfg(test)]
#[cfg(test)]
fn routing_policy(enabled: bool) -> AutoroutingPolicy {
    AutoroutingPolicy {
        enabled,
        bands: BTreeMap::from([
            (
                ComplexityBand::Low,
                RoutingBand {
                    model_id: None,
                    effort: "low".into(),
                    approval_required: false,
                    when_to_use: "Small, reversible tasks.".into(),
                },
            ),
            (
                ComplexityBand::Medium,
                RoutingBand {
                    model_id: Some("model-medium".into()),
                    effort: "high".into(),
                    approval_required: true,
                    when_to_use: "Changes with user-visible consequences.".into(),
                },
            ),
        ]),
    }
}

#[cfg(test)]
fn defaults() -> RoutingDefaults {
    RoutingDefaults {
        model_id: "default-model".into(),
        effort: "medium".into(),
    }
}

#[cfg(test)]
fn backup_config() -> RetainedBackupConfig {
    RetainedBackupConfig::new(
        "postgres://locus@localhost/locus",
        "/var/lib/locus/artifacts",
        "/var/lib/locus/backups",
    )
}

#[cfg(test)]
#[tokio::test]
async fn falls_up_and_records() {
    let (container, _cleanup) =
        crate::testkit::postgres::start_postgres_named("locus-routing-test").await;
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect store");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &crate::testkit::postgres::NoopMigrationBackup,
            &backup_config(),
        )
        .await
        .expect("migrate store");

    let project_id = ProjectId::generate();
    let agent_def_id = AgentDefId::generate();
    let session_id = SessionId::generate();
    let run_id = RunId::generate();
    query("INSERT INTO core.projects (id, name) VALUES ($1, 'routing test')")
        .bind(project_id)
        .execute(store.pool())
        .await
        .expect("insert project");
    query(
        "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'routing test agent', 1, '{}'::jsonb, '')",
    )
    .bind(agent_def_id)
    .execute(store.pool())
    .await
    .expect("insert agent definition");
    query(
        "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1, $2, $3, 'routing test session', 'agent/routing-test')",
    )
    .bind(session_id)
    .bind(project_id)
    .bind(agent_def_id)
    .execute(store.pool())
    .await
    .expect("insert session");
    query(
        "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
             VALUES ($1, $2, 'unrouted-model', 'queued')",
    )
    .bind(run_id)
    .bind(session_id)
    .execute(store.pool())
    .await
    .expect("insert run");

    let decision = routing_policy(true)
        .route(ComplexityBand::Low, &defaults())
        .expect("route low task");
    assert_eq!(decision.selected_band, Some(ComplexityBand::Medium));
    store
        .record_routing_decision(run_id, &decision)
        .await
        .expect("record routing decision");

    let row: (String, String, Option<String>, String, bool) = query_as(
        "SELECT resolved_model_id, routing_requested_band, routing_selected_band,
                    routing_effort, routing_approval_required
             FROM agents.runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(store.pool())
    .await
    .expect("load stored routing decision");
    assert_eq!(
            row,
            (
                "model-medium".into(),
                "low".into(),
                Some("medium".into()),
                "high".into(),
                true,
            ),
            "a missing low-band model falls upward and stores the selected band, model, effort, and approval state"
        );
}

#[cfg(test)]
#[test]
fn disabled_autorouting_uses_harness_defaults() {
    assert_eq!(
        routing_policy(false)
            .route(ComplexityBand::Max, &defaults())
            .expect("route with defaults"),
        RoutingDecision {
            requested_band: ComplexityBand::Max,
            selected_band: None,
            model_id: "default-model".into(),
            effort: "medium".into(),
            approval_required: false,
        }
    );
}
