//! Postgres as the single source of truth: the container, the pool, migrations, and
//! every query Locus runs. PLAN.md §Process topology names this `store`.
//!
//! This is the only sqlx-aware layer in the crate.

pub mod agents;
pub mod artifacts;
pub mod audits;
pub mod backup;
pub mod bus;
pub mod dispatch;
pub mod interact;
pub mod mail;
pub mod memory;
pub mod model_tiers;
pub mod planning;
pub mod projects;
pub mod providers;
pub mod qa;
pub mod restore;
pub mod routing;
pub mod schedules;
pub mod wiki;

use crate::ids::{EventId, RunId};
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};

use crate::{
    services::telemetry::Event,
    store::backup::{gate_migration, MigrationBackup, MigrationRunner, RetainedBackupConfig},
};
use tokio::{process::Command, time::sleep};

#[cfg(test)]
use crate::testkit::postgres::{test_backup_config, NoopMigrationBackup};

const POSTGRES_IMAGE: &str = "pgvector/pgvector:pg17";
const HEALTH_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Connection and storage settings for one local Postgres container.
///
/// The password is intentionally never included in command errors or logs.
pub struct PostgresConfig {
    container_name: String,
    volume_name: String,
    host_port: u16,
    user: String,
    database: String,
    password: String,
}

impl PostgresConfig {
    pub fn local(password: impl Into<String>, host_port: u16) -> Self {
        Self {
            container_name: "locus-postgres".into(),
            volume_name: "locus-postgres-data".into(),
            host_port,
            user: "locus".into(),
            database: "locus".into(),
            password: password.into(),
        }
    }

    /// Test-support constructor. `testkit::postgres` needs it, and integration tests
    /// under `tests/` reach that, so it cannot be `#[cfg(test)]`.
    pub fn for_test(container_name: String, volume_name: String, host_port: u16) -> Self {
        let password = format!("locus-{container_name}");

        Self {
            container_name,
            volume_name,
            host_port,
            user: "locus".into(),
            database: "locus".into(),
            password,
        }
    }
}

/// Controls the single pgvector-backed Postgres instance that Locus owns per machine.
pub struct PostgresContainer {
    config: PostgresConfig,
}

impl PostgresContainer {
    pub fn new(config: PostgresConfig) -> Self {
        Self { config }
    }

    /// Starts the container when absent or stopped, then waits until PostgreSQL reports healthy.
    pub async fn start(&self) -> Result<()> {
        self.validate_config()?;

        match self.state().await? {
            ContainerState::Missing => {
                self.docker(&[
                    "run".into(),
                    "--detach".into(),
                    "--name".into(),
                    self.config.container_name.clone(),
                    "--label".into(),
                    "com.locus.component=postgres".into(),
                    "--publish".into(),
                    format!("127.0.0.1:{}:5432", self.config.host_port),
                    "--volume".into(),
                    format!("{}:/var/lib/postgresql/data", self.config.volume_name),
                    "--env".into(),
                    format!("POSTGRES_USER={}", self.config.user),
                    "--env".into(),
                    format!("POSTGRES_DB={}", self.config.database),
                    "--env".into(),
                    format!("POSTGRES_PASSWORD={}", self.config.password),
                    "--health-cmd".into(),
                    format!(
                        "pg_isready -U {} -d {}",
                        self.config.user, self.config.database
                    ),
                    "--health-interval".into(),
                    "1s".into(),
                    "--health-timeout".into(),
                    "5s".into(),
                    "--health-retries".into(),
                    "20".into(),
                    POSTGRES_IMAGE.into(),
                ])
                .await?;
            }
            ContainerState::Stopped => {
                self.docker(&["start".into(), self.config.container_name.clone()])
                    .await?;
            }
            ContainerState::Healthy | ContainerState::Starting => {}
        }

        self.wait_for_healthy().await
    }

    /// Stops the database without removing its named volume.
    pub async fn stop(&self) -> Result<()> {
        match self.state().await? {
            ContainerState::Missing | ContainerState::Stopped => Ok(()),
            ContainerState::Healthy | ContainerState::Starting => {
                self.docker(&[
                    "stop".into(),
                    "--time".into(),
                    "10".into(),
                    self.config.container_name.clone(),
                ])
                .await?;
                Ok(())
            }
        }
    }

    pub async fn is_healthy(&self) -> Result<bool> {
        Ok(matches!(self.state().await?, ContainerState::Healthy))
    }

    /// The connection URL for this container. Integration tests under `tests/` reach it.
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@127.0.0.1:{}/{}?sslmode=disable",
            self.config.user, self.config.password, self.config.host_port, self.config.database
        )
    }

    async fn wait_for_healthy(&self) -> Result<()> {
        let deadline = Instant::now() + HEALTH_TIMEOUT;

        loop {
            match self.state().await? {
                ContainerState::Healthy => return Ok(()),
                ContainerState::Missing | ContainerState::Stopped => {
                    bail!("Postgres container stopped before it became healthy")
                }
                ContainerState::Starting if Instant::now() < deadline => {
                    sleep(HEALTH_POLL_INTERVAL).await;
                }
                ContainerState::Starting => {
                    bail!("Postgres container did not become healthy within 30 seconds")
                }
            }
        }
    }

    async fn state(&self) -> Result<ContainerState> {
        let output = Command::new("docker")
            .args([
                "container",
                "inspect",
                "--format",
                "{{if .State.Running}}{{.State.Health.Status}}{{else}}stopped{{end}}",
                &self.config.container_name,
            ])
            .output()
            .await
            .context("run docker container inspect")?;

        if output.status.success() {
            return match String::from_utf8_lossy(&output.stdout).trim() {
                "healthy" => Ok(ContainerState::Healthy),
                "starting" | "unhealthy" => Ok(ContainerState::Starting),
                "stopped" => Ok(ContainerState::Stopped),
                state => bail!("unexpected Docker health state `{state}`"),
            };
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such container") || stderr.contains("No such object") {
            Ok(ContainerState::Missing)
        } else {
            bail!("docker container inspect failed: {stderr}")
        }
    }

    async fn docker(&self, args: &[String]) -> Result<()> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .await
            .context("run docker")?;

        if output.status.success() {
            Ok(())
        } else {
            bail!(
                "docker command failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.host_port == 0 {
            bail!("Postgres host port must be non-zero")
        }
        if self.config.password.is_empty() {
            bail!("Postgres password must not be empty")
        }
        Ok(())
    }
}

/// The shared Postgres connection pool and migration runner.
#[derive(Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("connect to Postgres")?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(crate) fn connect_lazy(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_lazy(database_url)
            .context("create lazy Postgres pool")?;
        Ok(Self { pool })
    }

    /// Run the migrations shipped with this binary, behind the retained-backup gate.
    ///
    /// `sqlx::migrate!` embeds `migrations/` at compile time, so a deployed binary needs
    /// nothing on disk and a malformed migration fails the build rather than the boot.
    pub async fn run_embedded_migrations(
        &self,
        backup: &dyn MigrationBackup,
        backup_config: &RetainedBackupConfig,
    ) -> Result<()> {
        let migration = EmbeddedMigrationRunner { pool: &self.pool };
        gate_migration(backup, backup_config, &migration, Path::new("migrations")).await
    }

    /// Run migrations from a directory. Tests use this to point at a scratch tree; the
    /// deployed path is [`Store::run_embedded_migrations`].
    pub async fn run_migrations(
        &self,
        directory: impl AsRef<Path>,
        backup: &dyn MigrationBackup,
        backup_config: &RetainedBackupConfig,
    ) -> Result<()> {
        let migration = SqlxMigrationRunner { pool: &self.pool };
        gate_migration(backup, backup_config, &migration, directory.as_ref()).await
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Exposes the driver only to black-box integration tests that assert migrations.
    #[doc(hidden)]
    pub fn test_pool(&self) -> &PgPool {
        &self.pool
    }

    /// Atomically reserves one run port in the durable agent schema.
    pub async fn allocate_run_port(&self, run_id: RunId) -> Result<u16> {
        for _ in 0..1_000 {
            let port = sqlx::query_scalar::<_, i32>(
                "WITH candidate AS (
                    SELECT candidate.port
                    FROM generate_series(43000, 43999) AS candidate(port)
                    WHERE NOT EXISTS (
                        SELECT 1 FROM agents.run_ports WHERE agents.run_ports.port = candidate.port
                    )
                    LIMIT 1
                 )
                 INSERT INTO agents.run_ports (run_id, port)
                 SELECT $1, port FROM candidate
                 ON CONFLICT DO NOTHING
                 RETURNING port",
            )
            .bind(run_id)
            .fetch_optional(&self.pool)
            .await
            .context("allocate durable run port")?;
            if let Some(port) = port {
                return u16::try_from(port).context("allocated port is outside u16 range");
            }
        }
        bail!("no Locus ports remain")
    }

    /// Releases a run's durable port reservation once its container is gone.
    pub async fn release_run_port(&self, run_id: RunId) -> Result<()> {
        sqlx::query("DELETE FROM agents.run_ports WHERE run_id = $1")
            .bind(run_id)
            .execute(&self.pool)
            .await
            .context("release durable run port")?;
        Ok(())
    }

    /// Persists normalized events and their untouched source records as one transaction.
    /// Sequence assignment happens in telemetry before this method is called.
    pub async fn persist_events<'a>(
        &self,
        events: impl IntoIterator<Item = (EventId, &'a Event)>,
    ) -> Result<()> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("start normalized telemetry event transaction")?;

        for (event_id, event) in events {
            let payload = serde_json::json!({
                "text": event.text,
                "tool": event.tool,
                "args": event.args,
                "usage": event.usage,
            });
            sqlx::query(
                "INSERT INTO agents.events (id, run_id, seq, ts, verb, payload, raw)
                 VALUES ($1, $2, $3, $4::timestamptz, $5, $6::jsonb, $7::jsonb)",
            )
            .bind(event_id)
            .bind(event.run_id)
            .bind(i64::try_from(event.seq).context("event sequence exceeds PostgreSQL BIGINT")?)
            .bind(&event.ts)
            .bind(event.verb.as_str())
            .bind(payload)
            .bind(&event.raw)
            .execute(&mut *transaction)
            .await
            .context("persist normalized telemetry event")?;
        }

        transaction
            .commit()
            .await
            .context("commit normalized telemetry events")?;
        Ok(())
    }

    pub async fn persist_event(&self, event_id: EventId, event: &Event) -> Result<()> {
        self.persist_events([(event_id, event)]).await
    }
}

struct SqlxMigrationRunner<'a> {
    pool: &'a PgPool,
}

struct EmbeddedMigrationRunner<'a> {
    pool: &'a PgPool,
}

impl MigrationRunner for EmbeddedMigrationRunner<'_> {
    fn run_migrations<'a>(
        &'a self,
        _directory: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::migrate!("../../migrations")
                .run(self.pool)
                .await
                .context("run embedded SQLx migrations")
        })
    }
}

impl MigrationRunner for SqlxMigrationRunner<'_> {
    fn run_migrations<'a>(
        &'a self,
        directory: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            Migrator::new(directory)
                .await
                .context("load SQLx migrations")?
                .run(self.pool)
                .await
                .context("run SQLx migrations")
        })
    }
}

#[derive(Clone, Copy)]
enum ContainerState {
    Missing,
    Stopped,
    Starting,
    Healthy,
}

#[cfg(test)]
mod container_lifecycle {
    use std::process::Command;

    use super::{PostgresConfig, PostgresContainer};

    #[tokio::test]
    async fn container_lifecycle() {
        use crate::testkit::postgres::{serialize_postgres, unused_port, DockerCleanup};

        // This test drives start/stop itself, so it cannot use `start_postgres` — but it
        // still needs the same serialization, or a parallel container test collides.
        let _serialized = serialize_postgres().await;
        let suffix = format!("{}-{}", std::process::id(), unused_port());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup::new(container_name.clone(), volume_name.clone());
        let container = PostgresContainer::new(PostgresConfig::for_test(
            container_name.clone(),
            volume_name,
            unused_port(),
        ));

        container
            .start()
            .await
            .expect("start the pgvector container");
        assert!(container.is_healthy().await.expect("inspect health"));

        let vector_extension = Command::new("docker")
            .args([
                "exec",
                &container_name,
                "psql",
                "-qAt",
                "-U",
                "locus",
                "-d",
                "locus",
                "-v",
                "ON_ERROR_STOP=1",
                "-c",
                "CREATE EXTENSION IF NOT EXISTS vector; SELECT extname FROM pg_extension WHERE extname = 'vector'",
            ])
            .output()
            .expect("run psql in the container");
        assert!(
            vector_extension.status.success(),
            "pgvector query failed: {}",
            String::from_utf8_lossy(&vector_extension.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&vector_extension.stdout).trim(),
            "vector"
        );

        container.stop().await.expect("stop the container");
        assert!(!container
            .is_healthy()
            .await
            .expect("inspect stopped health"));

        container.start().await.expect("restart the container");
        assert!(container
            .is_healthy()
            .await
            .expect("inspect restarted health"));
    }
}

#[cfg(test)]
mod migrate_runs {
    use std::{fs, path::PathBuf, process::Command};

    use super::{PostgresConfig, PostgresContainer, Store};

    /// The migration test also writes a scratch migrations directory, so it needs a
    /// cleanup the shared fixture does not provide.
    struct MigrationCleanup {
        _container: crate::testkit::postgres::DockerCleanup,
        migrations_directory: PathBuf,
    }

    impl Drop for MigrationCleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.migrations_directory);
        }
    }

    #[tokio::test]
    async fn migrate_runs() {
        let port = crate::testkit::postgres::unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let migrations_directory = std::env::temp_dir().join(format!("locus-migrations-{suffix}"));
        fs::create_dir_all(&migrations_directory).expect("create migration directory");
        fs::write(
            migrations_directory.join("0001_create_migration_probe.sql"),
            "CREATE TABLE migration_probe (id INTEGER PRIMARY KEY);",
        )
        .expect("write migration");
        let _cleanup = MigrationCleanup {
            _container: crate::testkit::postgres::DockerCleanup::new(
                container_name.clone(),
                volume_name.clone(),
            ),
            migrations_directory: migrations_directory.clone(),
        };
        let container = PostgresContainer::new(PostgresConfig::for_test(
            container_name.clone(),
            volume_name,
            port,
        ));
        container
            .start()
            .await
            .expect("start the pgvector container");

        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the store pool");
        store
            .run_migrations(
                &migrations_directory,
                &super::NoopMigrationBackup,
                &super::test_backup_config(),
            )
            .await
            .expect("run the pending migration");

        let probe = Command::new("docker")
            .args([
                "exec",
                &container_name,
                "psql",
                "-qAt",
                "-U",
                "locus",
                "-d",
                "locus",
                "-c",
                "SELECT to_regclass('public.migration_probe')",
            ])
            .output()
            .expect("query the migrated schema");
        assert!(probe.status.success(), "query failed");
        assert_eq!(
            String::from_utf8_lossy(&probe.stdout).trim(),
            "migration_probe"
        );
    }
}

#[cfg(test)]
mod migrate_from_empty {

    use sqlx::query_scalar;

    use super::Store;

    #[tokio::test]
    async fn migrate_from_empty() {
        let (container, _cleanup) =
            crate::testkit::postgres::start_postgres_named("locus-postgres-test").await;

        let store = Store::connect(&container.database_url())
            .await
            .expect("connect the empty store pool");
        let schema_query = "
            SELECT schema_name
            FROM information_schema.schemata
            WHERE schema_name IN (
                'agents', 'board', 'core', 'mail', 'market', 'memory', 'wiki', 'workflows'
            )
            ORDER BY schema_name
        ";
        let schemas_before: Vec<String> = query_scalar(schema_query)
            .fetch_all(store.pool())
            .await
            .expect("query empty database schemas");
        assert!(
            schemas_before.is_empty(),
            "test database starts without Locus schemas"
        );

        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &super::NoopMigrationBackup,
                &super::test_backup_config(),
            )
            .await
            .expect("run all migrations once");

        let schemas_after: Vec<String> = query_scalar(schema_query)
            .fetch_all(store.pool())
            .await
            .expect("query migrated database schemas");
        assert_eq!(
            schemas_after,
            [
                "agents",
                "board",
                "core",
                "mail",
                "market",
                "memory",
                "wiki",
                "workflows"
            ],
            "one migration run creates every Locus schema"
        );
    }
}

#[cfg(test)]
mod migrations_reversible_or_explained {
    use std::{fs, path::Path};

    const ONE_WAY_REASON_PREFIX: &str = "-- one-way: ";

    #[test]
    fn migrations_reversible_or_explained() {
        // This regression starts green: every current schema migration has a down pair.
        let migrations_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
        let mut up_migrations: Vec<_> = fs::read_dir(&migrations_directory)
            .expect("read migrations directory")
            .map(|entry| entry.expect("read migration entry").path())
            .filter(|path| path.to_string_lossy().ends_with(".up.sql"))
            .collect();
        up_migrations.sort();
        assert!(
            !up_migrations.is_empty(),
            "repository contains up migrations"
        );

        for up_migration in up_migrations {
            let file_name = up_migration
                .file_name()
                .and_then(|name| name.to_str())
                .expect("migration filename is valid UTF-8");
            let down_migration = up_migration.with_file_name(
                file_name
                    .strip_suffix(".up.sql")
                    .expect("up migration has expected suffix")
                    .to_owned()
                    + ".down.sql",
            );

            if down_migration.is_file() {
                continue;
            }

            let contents = fs::read_to_string(&up_migration).expect("read one-way migration");
            let has_reason = contents.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(ONE_WAY_REASON_PREFIX)
                    .is_some_and(|reason| !reason.trim().is_empty())
            });
            assert!(
                has_reason,
                "{} needs {} or a `-- one-way: <reason>` comment",
                up_migration.display(),
                down_migration.display()
            );
        }
    }
}
