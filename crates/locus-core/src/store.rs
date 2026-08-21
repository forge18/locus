//! Lifecycle management for the machine-wide `locus-postgres` container.

use std::{
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};
use tokio::{process::Command, time::sleep};

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

    #[cfg(test)]
    fn for_test(container_name: String, volume_name: String, host_port: u16) -> Self {
        Self {
            container_name,
            volume_name,
            host_port,
            user: "locus".into(),
            database: "locus".into(),
            password: "test-password".into(),
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

    pub async fn run_migrations(&self, directory: impl AsRef<Path>) -> Result<()> {
        Migrator::new(directory.as_ref())
            .await
            .context("load SQLx migrations")?
            .run(&self.pool)
            .await
            .context("run SQLx migrations")
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
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
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use super::{PostgresConfig, PostgresContainer};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn container_lifecycle() {
        let suffix = format!("{}-{}", std::process::id(), unused_port());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
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
    use std::{
        fs,
        net::TcpListener,
        path::PathBuf,
        process::{Command, Stdio},
    };

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
        migrations_directory: PathBuf,
    }

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
            let _ = fs::remove_dir_all(&self.migrations_directory);
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn migrate_runs() {
        let port = unused_port();
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
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
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

        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(&migrations_directory)
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
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn migrate_from_empty() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the empty pgvector container");

        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
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

#[cfg(test)]
mod schema_core {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_core() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the core migration");

        for table in ["projects", "repos", "local_remotes", "settings"] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'core' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the core schema");
            assert!(exists, "core.{table} exists");
        }
    }
}

#[cfg(test)]
mod schema_agents {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_agents() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the agents migration");

        for table in [
            "agent_defs",
            "sessions",
            "runs",
            "run_edges",
            "events",
            "artifacts",
            "artifact_comments",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'agents' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the agents schema");
            assert!(exists, "agents.{table} exists");
        }
    }
}

#[cfg(test)]
mod schema_board {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_board() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the board migration");

        for table in [
            "tasks",
            "task_dependencies",
            "task_transitions",
            "task_assignments",
            "task_runs",
            "task_evidence",
            "github_issues",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'board' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the board schema");
            assert!(exists, "board.{table} exists");
        }
    }
}

#[cfg(test)]
mod schema_wiki {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_wiki() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the wiki migration");

        for table in [
            "pages",
            "revisions",
            "links",
            "contradictions",
            "ingest_log",
            "embeddings",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'wiki' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the wiki schema");
            assert!(exists, "wiki.{table} exists");
        }

        let vector_available: bool =
            query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
                .fetch_one(store.pool())
                .await
                .expect("query the vector extension");
        assert!(vector_available, "pgvector is enabled for wiki embeddings");
    }
}

#[cfg(test)]
mod schema_memory {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_memory() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the memory migration");

        for table in ["core", "store", "probation", "edges"] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'memory' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the memory schema");
            assert!(exists, "memory.{table} exists");
        }

        for column in [
            "project_id",
            "agent_def_id",
            "scope",
            "provenance",
            "embedding",
            "confidence",
            "importance",
            "recall_count",
            "active_days",
            "strength",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'memory' AND table_name = 'store' AND column_name = $1
                )",
            )
            .bind(column)
            .fetch_one(store.pool())
            .await
            .expect("query durable memory columns");
            assert!(exists, "memory.store.{column} exists");
        }

        let embedding_type: String = query_scalar(
            "SELECT udt_name
                FROM information_schema.columns
                WHERE table_schema = 'memory' AND table_name = 'store' AND column_name = 'embedding'",
        )
        .fetch_one(store.pool())
        .await
        .expect("query the durable memory embedding type");
        assert_eq!(
            embedding_type, "vector",
            "memory.store.embedding uses pgvector"
        );

        for index in [
            "memory_core_project_agent_idx",
            "memory_store_project_scope_idx",
            "memory_store_project_path_idx",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM pg_indexes
                    WHERE schemaname = 'memory' AND indexname = $1
                )",
            )
            .bind(index)
            .fetch_one(store.pool())
            .await
            .expect("query memory indexes");
            assert!(exists, "memory index {index} exists");
        }
    }
}

#[cfg(test)]
mod schema_workflows {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::{query, query_scalar};

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_workflows() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the workflows migration");

        for table in [
            "workflow_defs",
            "schedules",
            "executions",
            "iterations",
            "guardrail_trips",
            "verify_results",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'workflows' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the workflows schema");
            assert!(exists, "workflows.{table} exists");
        }

        for column in [
            "project_id",
            "name",
            "version",
            "graph",
            "spec",
            "verify_command",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'workflows'
                        AND table_name = 'workflow_defs'
                        AND column_name = $1
                )",
            )
            .bind(column)
            .fetch_one(store.pool())
            .await
            .expect("query workflow definition columns");
            assert!(exists, "workflows.workflow_defs.{column} exists");
        }

        for (table, column) in [
            ("schedules", "cron_expression"),
            ("schedules", "paused_at"),
            ("executions", "status"),
            ("executions", "schedule_id"),
            ("iterations", "arbiter_class"),
            ("iterations", "counts_toward_iteration_budget"),
            ("guardrail_trips", "guardrail"),
            ("verify_results", "passed"),
            ("verify_results", "exit_code"),
            ("verify_results", "stdout"),
            ("verify_results", "stderr"),
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'workflows' AND table_name = $1 AND column_name = $2
                )",
            )
            .bind(table)
            .bind(column)
            .fetch_one(store.pool())
            .await
            .expect("query workflow lifecycle columns");
            assert!(exists, "workflows.{table}.{column} exists");
        }

        for index in [
            "workflow_defs_project_name_version_key",
            "schedules_workflow_def_id_idx",
            "executions_schedule_id_idx",
            "executions_active_schedule_idx",
            "iterations_execution_number_key",
            "guardrail_trips_execution_id_idx",
            "verify_results_execution_id_idx",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM pg_indexes
                    WHERE schemaname = 'workflows' AND indexname = $1
                )",
            )
            .bind(index)
            .fetch_one(store.pool())
            .await
            .expect("query workflow indexes");
            assert!(exists, "workflow index {index} exists");
        }

        query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'workflow test project')")
            .bind("00000000-0000-0000-0000-000000000001")
            .execute(store.pool())
            .await
            .expect("insert workflow test project");
        query(
            "INSERT INTO workflows.workflow_defs
                (id, project_id, name, version, graph, spec, verify_command)
             VALUES
                ($1::uuid, $2::uuid, 'test workflow', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .bind("00000000-0000-0000-0000-000000000002")
        .bind("00000000-0000-0000-0000-000000000001")
        .execute(store.pool())
        .await
        .expect("insert immutable workflow definition");
        let update = query(
            "UPDATE workflows.workflow_defs SET name = 'changed workflow' WHERE id = $1::uuid",
        )
        .bind("00000000-0000-0000-0000-000000000002")
        .execute(store.pool())
        .await;
        assert!(update.is_err(), "workflow definitions are immutable");
    }
}

#[cfg(test)]
mod schema_mail {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::{query, query_scalar};

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_mail() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the mail migration");

        for table in ["threads", "messages", "deliveries", "waits"] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'mail' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the mail schema");
            assert!(exists, "mail.{table} exists");
        }

        for (table, column) in [
            ("threads", "project_id"),
            ("threads", "subject"),
            ("messages", "thread_id"),
            ("messages", "sender_kind"),
            ("messages", "sender_run_id"),
            ("messages", "body"),
            ("deliveries", "message_id"),
            ("deliveries", "recipient_kind"),
            ("deliveries", "recipient_session_id"),
            ("deliveries", "status"),
            ("waits", "run_id"),
            ("waits", "reason"),
            ("waits", "started_at"),
            ("waits", "ended_at"),
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'mail' AND table_name = $1 AND column_name = $2
                )",
            )
            .bind(table)
            .bind(column)
            .fetch_one(store.pool())
            .await
            .expect("query mail schema columns");
            assert!(exists, "mail.{table}.{column} exists");
        }

        for index in [
            "mail_threads_project_id_idx",
            "mail_messages_thread_id_idx",
            "mail_deliveries_recipient_session_pending_idx",
            "mail_waits_active_run_key",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM pg_indexes
                    WHERE schemaname = 'mail' AND indexname = $1
                )",
            )
            .bind(index)
            .fetch_one(store.pool())
            .await
            .expect("query mail indexes");
            assert!(exists, "mail index {index} exists");
        }

        query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'mail test project')")
            .bind("00000000-0000-0000-0000-000000000101")
            .execute(store.pool())
            .await
            .expect("insert mail test project");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, 'mail test agent', 1, '{}'::jsonb, 'test agent')",
        )
        .bind("00000000-0000-0000-0000-000000000102")
        .execute(store.pool())
        .await
        .expect("insert mail test agent");
        query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'mail test session', 'agent/mail-test')",
        )
        .bind("00000000-0000-0000-0000-000000000103")
        .bind("00000000-0000-0000-0000-000000000101")
        .bind("00000000-0000-0000-0000-000000000102")
        .execute(store.pool())
        .await
        .expect("insert mail test session");
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
             VALUES ($1::uuid, $2::uuid, 'test-model', 'running')",
        )
        .bind("00000000-0000-0000-0000-000000000104")
        .bind("00000000-0000-0000-0000-000000000103")
        .execute(store.pool())
        .await
        .expect("insert mail test run");

        query(
            "INSERT INTO mail.threads (id, project_id, subject)
             VALUES ($1::uuid, $2::uuid, 'mail test thread')",
        )
        .bind("00000000-0000-0000-0000-000000000105")
        .bind("00000000-0000-0000-0000-000000000101")
        .execute(store.pool())
        .await
        .expect("insert project-scoped mail thread");
        query(
            "INSERT INTO mail.messages (id, thread_id, sender_kind, sender_run_id, body)
             VALUES ($1::uuid, $2::uuid, 'agent', $3::uuid, 'mail test message')",
        )
        .bind("00000000-0000-0000-0000-000000000106")
        .bind("00000000-0000-0000-0000-000000000105")
        .bind("00000000-0000-0000-0000-000000000104")
        .execute(store.pool())
        .await
        .expect("insert threaded mail message");
        query(
            "INSERT INTO mail.deliveries (id, message_id, recipient_kind, recipient_session_id)
             VALUES ($1::uuid, $2::uuid, 'agent', $3::uuid)",
        )
        .bind("00000000-0000-0000-0000-000000000107")
        .bind("00000000-0000-0000-0000-000000000106")
        .bind("00000000-0000-0000-0000-000000000103")
        .execute(store.pool())
        .await
        .expect("insert agent delivery state");
        query(
            "INSERT INTO mail.waits (id, run_id, reason)
             VALUES ($1::uuid, $2::uuid, 'mail')",
        )
        .bind("00000000-0000-0000-0000-000000000108")
        .bind("00000000-0000-0000-0000-000000000104")
        .execute(store.pool())
        .await
        .expect("record mail wait reason");

        let second_active_wait = query(
            "INSERT INTO mail.waits (id, run_id, reason)
             VALUES ($1::uuid, $2::uuid, 'ask')",
        )
        .bind("00000000-0000-0000-0000-000000000109")
        .bind("00000000-0000-0000-0000-000000000104")
        .execute(store.pool())
        .await;
        assert!(
            second_active_wait.is_err(),
            "a run has at most one active wait"
        );

        let waiting_status = query("UPDATE agents.runs SET status = 'waiting' WHERE id = $1::uuid")
            .bind("00000000-0000-0000-0000-000000000104")
            .execute(store.pool())
            .await;
        assert!(
            waiting_status.is_err(),
            "waiting reasons are stored in mail.waits, not agents.runs.status"
        );

        query("DELETE FROM agents.runs WHERE id = $1::uuid")
            .bind("00000000-0000-0000-0000-000000000104")
            .execute(store.pool())
            .await
            .expect("delete a finished sender run");
        let sender_run_id: Option<String> =
            query_scalar("SELECT sender_run_id::text FROM mail.messages WHERE id = $1::uuid")
                .bind("00000000-0000-0000-0000-000000000106")
                .fetch_one(store.pool())
                .await
                .expect("read preserved mail message");
        assert_eq!(
            sender_run_id, None,
            "mail remains durable when its sender run is removed"
        );
    }
}

#[cfg(test)]
mod schema_market {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::{query, query_scalar};

    use super::{PostgresConfig, PostgresContainer, Store};

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

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

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[tokio::test]
    async fn schema_market() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-postgres-test-{suffix}");
        let volume_name = format!("locus-postgres-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container
            .start()
            .await
            .expect("start the pgvector container");
        let store = Store::connect(&format!(
            "postgres://locus:test-password@127.0.0.1:{port}/locus"
        ))
        .await
        .expect("connect the store pool");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            )
            .await
            .expect("run the market migration");

        for table in [
            "manifest_snapshots",
            "installs",
            "tool_sets",
            "tool_set_manifest_pins",
            "agent_tool_set_resolutions",
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_schema = 'market' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query the market schema");
            assert!(exists, "market.{table} exists");
        }

        for (table, column) in [
            ("manifest_snapshots", "name"),
            ("manifest_snapshots", "manifest"),
            ("manifest_snapshots", "content_sha256"),
            ("installs", "tool_set_id"),
            ("installs", "manifest_snapshot_id"),
            ("installs", "status"),
            ("tool_sets", "base_image_digest"),
            ("tool_sets", "image_cache_key"),
            ("tool_set_manifest_pins", "tool_name"),
            ("tool_set_manifest_pins", "manifest_snapshot_id"),
            ("agent_tool_set_resolutions", "agent_def_id"),
            ("agent_tool_set_resolutions", "tool_set_id"),
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'market' AND table_name = $1 AND column_name = $2
                )",
            )
            .bind(table)
            .bind(column)
            .fetch_one(store.pool())
            .await
            .expect("query market schema columns");
            assert!(exists, "market.{table}.{column} exists");
        }

        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1::uuid, 'market test agent', 1, '{}'::jsonb, 'test agent')",
        )
        .bind("00000000-0000-0000-0000-000000000201")
        .execute(store.pool())
        .await
        .expect("insert market test agent");
        query(
            "INSERT INTO market.manifest_snapshots (id, name, manifest, content_sha256)
             VALUES ($1::uuid, 'amq',
                     '{\"name\": \"amq\", \"install\": {\"cargo\": \"agent-message-queue\"}}'::jsonb,
                     'a3f7b1')",
        )
        .bind("00000000-0000-0000-0000-000000000202")
        .execute(store.pool())
        .await
        .expect("cache a manifest snapshot");
        query(
            "INSERT INTO market.tool_sets (id, base_image_digest, image_cache_key)
             VALUES ($1::uuid, 'sha256:base-image', 'sha256:agent-image')",
        )
        .bind("00000000-0000-0000-0000-000000000203")
        .execute(store.pool())
        .await
        .expect("create deterministic image tool set");
        query(
            "INSERT INTO market.tool_set_manifest_pins (tool_set_id, tool_name, manifest_snapshot_id)
             VALUES ($1::uuid, 'amq', $2::uuid)",
        )
        .bind("00000000-0000-0000-0000-000000000203")
        .bind("00000000-0000-0000-0000-000000000202")
        .execute(store.pool())
        .await
        .expect("pin manifest snapshot in tool set");
        query(
            "INSERT INTO market.installs (id, tool_set_id, manifest_snapshot_id, status)
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'verified')",
        )
        .bind("00000000-0000-0000-0000-000000000204")
        .bind("00000000-0000-0000-0000-000000000203")
        .bind("00000000-0000-0000-0000-000000000202")
        .execute(store.pool())
        .await
        .expect("record an install for a pinned manifest");
        query(
            "INSERT INTO market.agent_tool_set_resolutions (agent_def_id, tool_set_id)
             VALUES ($1::uuid, $2::uuid)",
        )
        .bind("00000000-0000-0000-0000-000000000201")
        .bind("00000000-0000-0000-0000-000000000203")
        .execute(store.pool())
        .await
        .expect("resolve the agent definition to its tool set");

        let duplicate_cache_key = query(
            "INSERT INTO market.tool_sets (id, base_image_digest, image_cache_key)
             VALUES ($1::uuid, 'sha256:base-image', 'sha256:agent-image')",
        )
        .bind("00000000-0000-0000-0000-000000000205")
        .execute(store.pool())
        .await;
        assert!(
            duplicate_cache_key.is_err(),
            "one deterministic cache key identifies one tool set"
        );

        let duplicate_resolution = query(
            "INSERT INTO market.agent_tool_set_resolutions (agent_def_id, tool_set_id)
             VALUES ($1::uuid, $2::uuid)",
        )
        .bind("00000000-0000-0000-0000-000000000201")
        .bind("00000000-0000-0000-0000-000000000203")
        .execute(store.pool())
        .await;
        assert!(
            duplicate_resolution.is_err(),
            "an immutable agent definition has one resolved tool set"
        );

        let mismatched_pin = query(
            "INSERT INTO market.tool_set_manifest_pins (tool_set_id, tool_name, manifest_snapshot_id)
             VALUES ($1::uuid, 'not-amq', $2::uuid)",
        )
        .bind("00000000-0000-0000-0000-000000000203")
        .bind("00000000-0000-0000-0000-000000000202")
        .execute(store.pool())
        .await;
        assert!(
            mismatched_pin.is_err(),
            "a tool-set pin must use the snapshot's tool name"
        );
    }
}
