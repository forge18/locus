//! The Docker daemon endpoint, resolved from the active context.

use super::*;

/// A narrow bollard wrapper. The daemon remains the sole Docker authority; no agent gets it.
#[derive(Clone)]
pub struct DockerDaemon {
    docker: Docker,
}

impl DockerDaemon {
    pub fn connect() -> Result<Self> {
        let endpoint = std::env::var("DOCKER_HOST")
            .ok()
            .or_else(active_context_endpoint);
        let docker = match endpoint.as_deref() {
            Some(endpoint) if endpoint.starts_with("unix://") => {
                Docker::connect_with_socket(endpoint, 120, API_DEFAULT_VERSION)
            }
            _ => Docker::connect_with_defaults(),
        }
        .context("connect to Docker daemon")?;
        Ok(Self { docker })
    }

    pub async fn ping(&self) -> Result<()> {
        self.docker.ping().await.context("ping Docker daemon")?;
        Ok(())
    }

    /// Start a previously-created container through the host-only daemon client.
    pub async fn start(&self, container: &str) -> Result<()> {
        self.docker
            .start_container(container, None::<StartContainerOptions>)
            .await
            .with_context(|| format!("start container `{container}`"))
    }

    /// Stop one container without exposing the daemon socket to an agent.
    pub async fn stop(&self, container: &str) -> Result<()> {
        self.docker
            .stop_container(container, None::<StopContainerOptions>)
            .await
            .with_context(|| format!("stop container `{container}`"))
    }

    /// Remove one finished container through the same host-only daemon client.
    pub async fn remove(&self, container: &str) -> Result<()> {
        self.docker
            .remove_container(container, None::<RemoveContainerOptions>)
            .await
            .with_context(|| format!("remove container `{container}`"))
    }
}

fn active_context_endpoint() -> Option<String> {
    let output = Command::new("docker")
        .args([
            "context",
            "inspect",
            "--format",
            "{{.Endpoints.docker.Host}}",
        ])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|endpoint| !endpoint.is_empty())
}

#[cfg(test)]
mod endpoint {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn connects() {
        DockerDaemon::connect().unwrap().ping().await.unwrap();
    }
}
