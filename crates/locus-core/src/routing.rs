//! Six-band autorouting policy and its durable run decision.

use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sqlx::query;
use uuid::Uuid;

use crate::store::Store;

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

/// The model and effort selected whenever autorouting is disabled.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingDefaults {
    pub model_id: String,
    pub effort: String,
}

/// Settings for one complexity band. A missing `model_id` deliberately falls upward.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingBand {
    pub model_id: Option<String>,
    pub effort: String,
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
    pub effort: String,
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
            require_nonempty("default effort", &defaults.effort)?;
            return Ok(RoutingDecision {
                requested_band,
                selected_band: None,
                model_id: defaults.model_id.clone(),
                effort: defaults.effort.clone(),
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
            require_nonempty("band effort", &band.effort)?;
            require_nonempty("band when-to-use prose", &band.when_to_use)?;
            return Ok(RoutingDecision {
                requested_band,
                selected_band: Some(selected_band),
                model_id: model_id.clone(),
                effort: band.effort.clone(),
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

impl Store {
    /// Store the routing selection alongside the actual model that answers a run.
    pub async fn record_routing_decision(
        &self,
        run_id: Uuid,
        decision: &RoutingDecision,
    ) -> Result<()> {
        let updated = query(
            "UPDATE agents.runs
             SET resolved_model_id = $2,
                 routing_requested_band = $3,
                 routing_selected_band = $4,
                 routing_effort = $5,
                 routing_approval_required = $6
             WHERE id = $1",
        )
        .bind(run_id)
        .bind(&decision.model_id)
        .bind(band_name(decision.requested_band))
        .bind(decision.selected_band.map(band_name))
        .bind(&decision.effort)
        .bind(decision.approval_required)
        .execute(self.pool())
        .await?;
        if updated.rows_affected() != 1 {
            bail!("run `{run_id}` does not exist")
        }
        Ok(())
    }
}

fn band_name(band: ComplexityBand) -> &'static str {
    match band {
        ComplexityBand::XtraLow => "xtra-low",
        ComplexityBand::Low => "low",
        ComplexityBand::Medium => "medium",
        ComplexityBand::High => "high",
        ComplexityBand::XtraHigh => "xtra-high",
        ComplexityBand::Max => "max",
    }
}

#[cfg(test)]
use std::{
    net::TcpListener,
    process::{Command, Stdio},
};

#[cfg(test)]
use crate::{
    backup::{MigrationBackup, RetainedBackupConfig},
    store::{PostgresConfig, PostgresContainer},
};
#[cfg(test)]
use sqlx::query_as;

#[cfg(test)]
struct NoopMigrationBackup;

#[cfg(test)]
impl MigrationBackup for NoopMigrationBackup {
    fn create_retained(&self, _: &RetainedBackupConfig) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
struct DockerCleanup {
    container_name: String,
    volume_name: String,
}

#[cfg(test)]
impl Drop for DockerCleanup {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "--force", &self.container_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = Command::new("docker")
            .args(["volume", "rm", "--force", &self.volume_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

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
fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
    listener.local_addr().expect("read local port").port()
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
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-routing-test-{suffix}");
    let volume_name = format!("locus-routing-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container.start().await.expect("start PostgreSQL");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect store");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &backup_config(),
        )
        .await
        .expect("migrate store");

    let project_id = Uuid::new_v4();
    let agent_def_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
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
