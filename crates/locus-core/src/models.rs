//! Project-scoped model tier settings.

use anyhow::{bail, Result};
use sqlx::query_scalar;

use crate::{registry::HarnessDefinition, store::Store};

#[cfg(test)]
use std::{
    net::TcpListener,
    process::{Command, Stdio},
};

#[cfg(test)]
use sqlx::query;

#[cfg(test)]
use crate::{
    backup::{MigrationBackup, RetainedBackupConfig},
    store::{PostgresConfig, PostgresContainer},
};

/// Resolve a requested model tier to its configured model through the permitted fallback order.
///
/// `None` leaves model selection to the harness's own default.
pub async fn resolve_tier(
    store: &Store,
    project_id: &str,
    harness_name: &str,
    requested_tier: &str,
) -> Result<Option<String>> {
    let fallback_tiers: &[&str] = match requested_tier {
        "xhigh" => &["xhigh"],
        "high" => &["high", "xhigh"],
        "medium" => &["medium", "high", "xhigh"],
        "low" => &["low", "medium", "high", "xhigh"],
        tier => bail!("unknown model tier `{tier}`"),
    };

    query_scalar(
        "SELECT model_id
         FROM core.model_tier_settings
         WHERE project_id = $1::uuid
           AND harness_name = $2
           AND tier = ANY($3::text[])
         ORDER BY array_position($3::text[], tier)
         LIMIT 1",
    )
    .bind(project_id)
    .bind(harness_name)
    .bind(fallback_tiers)
    .fetch_optional(store.pool())
    .await
    .map_err(Into::into)
}

/// Build the harness arguments for a resolved model, preserving its default when unset.
pub fn launch_argv(harness: &HarnessDefinition, model_id: Option<&str>) -> Vec<String> {
    let mut argv = harness.launch.argv.clone();
    if let Some(model_id) = model_id {
        argv.push(harness.models.flag.clone());
        argv.push(model_id.into());
    }
    argv
}

#[cfg(test)]
struct NoopMigrationBackup;

#[cfg(test)]
impl MigrationBackup for NoopMigrationBackup {
    fn create_retained(&self, _: &RetainedBackupConfig) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
fn test_backup_config() -> RetainedBackupConfig {
    RetainedBackupConfig::new(
        "postgres://locus@localhost/locus",
        "/var/lib/locus/artifacts",
        "/var/lib/locus/backups",
    )
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
fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
    listener.local_addr().expect("read the local port").port()
}

#[cfg(test)]
#[tokio::test]
async fn falls_back_up() {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-model-resolution-test-{suffix}");
    let volume_name = format!("locus-model-resolution-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container
        .start()
        .await
        .expect("start the model resolution test container");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("run migrations");

    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'model resolution test')")
        .bind("00000000-0000-0000-0000-000000000001")
        .execute(store.pool())
        .await
        .expect("insert project for model resolution");
    query(
        "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
         VALUES ($1::uuid, 'test-harness', 'xhigh', 'model-xhigh')",
    )
    .bind("00000000-0000-0000-0000-000000000001")
    .execute(store.pool())
    .await
    .expect("insert xhigh tier model");

    assert_eq!(
        resolve_tier(
            &store,
            "00000000-0000-0000-0000-000000000001",
            "test-harness",
            "high",
        )
        .await
        .expect("resolve configured model tier")
        .as_deref(),
        Some("model-xhigh"),
        "an unset high tier uses the configured xhigh tier"
    );
}

#[cfg(test)]
#[tokio::test]
async fn resolved_id_on_run() {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-model-run-test-{suffix}");
    let volume_name = format!("locus-model-run-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container
        .start()
        .await
        .expect("start the resolved model run test container");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("run migrations");

    const PROJECT_ID: &str = "00000000-0000-0000-0000-000000000011";
    const AGENT_DEF_ID: &str = "00000000-0000-0000-0000-000000000012";
    const SESSION_ID: &str = "00000000-0000-0000-0000-000000000013";
    const RUN_ID: &str = "00000000-0000-0000-0000-000000000014";

    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'resolved model run test')")
        .bind(PROJECT_ID)
        .execute(store.pool())
        .await
        .expect("insert project");
    query(
        "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
         VALUES ($1::uuid, 'resolved model test agent', 1, '{}'::jsonb, 'test agent')",
    )
    .bind(AGENT_DEF_ID)
    .execute(store.pool())
    .await
    .expect("insert agent definition");
    query(
        "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
         VALUES ($1::uuid, $2::uuid, $3::uuid, 'resolved model test session', 'agent/model-test')",
    )
    .bind(SESSION_ID)
    .bind(PROJECT_ID)
    .bind(AGENT_DEF_ID)
    .execute(store.pool())
    .await
    .expect("insert session");
    query(
        "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
         VALUES ($1::uuid, 'test-harness', 'high', 'model-high')",
    )
    .bind(PROJECT_ID)
    .execute(store.pool())
    .await
    .expect("insert model tier setting");

    let resolved_model_id = resolve_tier(&store, PROJECT_ID, "test-harness", "high")
        .await
        .expect("resolve configured model tier")
        .expect("configured high tier resolves to a model id");
    query(
        "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
         VALUES ($1::uuid, $2::uuid, $3, 'queued')",
    )
    .bind(RUN_ID)
    .bind(SESSION_ID)
    .bind(&resolved_model_id)
    .execute(store.pool())
    .await
    .expect("record run with its resolved model id");

    let recorded_model_id: String =
        query_scalar("SELECT resolved_model_id FROM agents.runs WHERE id = $1::uuid")
            .bind(RUN_ID)
            .fetch_one(store.pool())
            .await
            .expect("read the run's resolved model id");
    assert_eq!(recorded_model_id, resolved_model_id);
}

#[cfg(test)]
#[tokio::test]
async fn never_falls_down() {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-model-no-downward-test-{suffix}");
    let volume_name = format!("locus-model-no-downward-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container
        .start()
        .await
        .expect("start the model no-downward test container");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("run migrations");

    const PROJECT_ID: &str = "00000000-0000-0000-0000-000000000001";
    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'model no-downward test')")
        .bind(PROJECT_ID)
        .execute(store.pool())
        .await
        .expect("insert project for model no-downward resolution");

    for (requested_tier, weaker_tier) in [("xhigh", "high"), ("high", "medium"), ("medium", "low")]
    {
        let harness_name = format!("test-harness-{requested_tier}");
        query(
            "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
             VALUES ($1::uuid, $2, $3, 'weaker-model')",
        )
        .bind(PROJECT_ID)
        .bind(&harness_name)
        .bind(weaker_tier)
        .execute(store.pool())
        .await
        .expect("insert weaker tier model");

        assert_eq!(
            resolve_tier(&store, PROJECT_ID, &harness_name, requested_tier)
                .await
                .expect("resolve without weaker fallback"),
            None,
            "a requested {requested_tier} tier must not resolve to weaker {weaker_tier}"
        );
    }
}

#[cfg(test)]
#[test]
fn unset_uses_harness_default() {
    let mut harness: HarnessDefinition =
        toml::from_str(include_str!("../../../harnesses/claude.toml"))
            .expect("reference harness definition parses");
    harness.binary = "sh".into();
    harness.launch.argv = vec!["-c".into(), "exit 0".into()];

    let argv = launch_argv(&harness, None);
    assert_eq!(
        argv, harness.launch.argv,
        "an unset model tier preserves the harness's launch arguments"
    );
    assert!(
        !argv.iter().any(|argument| argument == &harness.models.flag),
        "an unset model tier passes no model flag"
    );

    let status = Command::new(&harness.binary)
        .args(argv)
        .status()
        .expect("start the harness with its default model");
    assert!(
        status.success(),
        "the harness starts with its default model"
    );
}

#[cfg(test)]
#[tokio::test]
async fn settings_table() {
    let port = unused_port();
    let suffix = format!("{}-{port}", std::process::id());
    let container_name = format!("locus-model-settings-test-{suffix}");
    let volume_name = format!("locus-model-settings-test-data-{suffix}");
    let _cleanup = DockerCleanup {
        container_name: container_name.clone(),
        volume_name: volume_name.clone(),
    };
    let container =
        PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
    container
        .start()
        .await
        .expect("start the model settings test container");
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &NoopMigrationBackup,
            &test_backup_config(),
        )
        .await
        .expect("run migrations");

    let columns: Vec<String> = query_scalar(
        "SELECT column_name
         FROM information_schema.columns
         WHERE table_schema = 'core' AND table_name = 'model_tier_settings'
         ORDER BY ordinal_position",
    )
    .fetch_all(store.pool())
    .await
    .expect("query model tier setting columns");
    assert_eq!(
        columns,
        [
            "project_id",
            "harness_name",
            "tier",
            "model_id",
            "updated_at"
        ]
    );

    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'model settings test')")
        .bind("00000000-0000-0000-0000-000000000001")
        .execute(store.pool())
        .await
        .expect("insert project for model settings");
    query(
        "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
         VALUES ($1::uuid, 'test-harness', 'high', 'model-high')",
    )
    .bind("00000000-0000-0000-0000-000000000001")
    .execute(store.pool())
    .await
    .expect("insert model tier mapping");

    let duplicate = query(
        "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
         VALUES ($1::uuid, 'test-harness', 'high', 'another-model')",
    )
    .bind("00000000-0000-0000-0000-000000000001")
    .execute(store.pool())
    .await;
    assert!(
        duplicate.is_err(),
        "a project may map each harness and tier only once"
    );

    let invalid_tier = query(
        "INSERT INTO core.model_tier_settings (project_id, harness_name, tier, model_id)
         VALUES ($1::uuid, 'test-harness', 'invalid', 'model-invalid')",
    )
    .bind("00000000-0000-0000-0000-000000000001")
    .execute(store.pool())
    .await;
    assert!(
        invalid_tier.is_err(),
        "only the four model tiers are accepted"
    );

    query("DELETE FROM core.projects WHERE id = $1::uuid")
        .bind("00000000-0000-0000-0000-000000000001")
        .execute(store.pool())
        .await
        .expect("delete project");
    let remaining: i64 = query_scalar("SELECT count(*) FROM core.model_tier_settings")
        .fetch_one(store.pool())
        .await
        .expect("count cascaded model tier mappings");
    assert_eq!(
        remaining, 0,
        "model tier mappings cascade with their project"
    );
}
