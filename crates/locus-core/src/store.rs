//! Lifecycle management for the machine-wide `locus-postgres` container.

use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
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
