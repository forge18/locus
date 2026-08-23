//! The container supervisor: bollard over the Docker Engine API, and the PTY stream.

use crate::bus::InProcessBus;
use anyhow::{bail, Context, Result};
use bollard::{
    container::LogOutput,
    models::{ContainerCreateBody, HostConfig},
    query_parameters::{
        AttachContainerOptionsBuilder, BuildImageOptionsBuilder, CreateContainerOptionsBuilder,
        StartContainerOptions, StopContainerOptions,
    },
    Docker,
};
use futures::StreamExt;
use tokio::sync::broadcast;

use crate::sandbox::{mounts::Mount, mounts::PtyAttachment};
/// Whether the container runtime built the image or reused its existing cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDisposition {
    Built,
    Reused,
}

pub(crate) const PTY_STREAM_CAPACITY: usize = 1_024;

/// Broadcasts raw PTY bytes from a run's container runtime to its UI subscribers.
#[derive(Clone, Debug)]
pub struct PtyStream(InProcessBus<Vec<u8>>);

impl PtyStream {
    pub fn new(capacity: usize) -> Self {
        Self(InProcessBus::new(capacity))
    }

    /// Registers one UI consumer. The desktop forwards each received buffer through
    /// its `Channel<&[u8]>` transport.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.0.subscribe()
    }

    /// Delivers one byte buffer read from the attached PTY.
    pub fn write(&self, bytes: &[u8]) -> usize {
        self.0.publish(bytes.to_vec())
    }
}

impl PartialEq for PtyStream {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_channel(&other.0)
    }
}

impl Eq for PtyStream {}

/// The complete, harness-agnostic request made to the container runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerLaunch {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub entrypoint: String,
    pub environment: Vec<String>,
    pub mounts: Vec<Mount>,
    pub network: String,
}

/// The narrow container boundary required by run spawning.
///
/// The supplied container adapter owns image caching, container creation, and PTY plumbing; this
/// supervisor owns their ordering and the run state transition.
pub trait ContainerRuntime {
    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition>;
    fn start_container(&mut self, container: &ContainerLaunch) -> Result<()>;
    fn attach_pty(
        &mut self,
        container: &str,
        attachment: PtyAttachment,
        stream: PtyStream,
    ) -> Result<()>;
    fn stop_container(&mut self, container: &str) -> Result<()>;
}

/// Host-only Bollard adapter. Agents never receive the Docker client or its socket.
#[derive(Clone)]
pub struct DockerContainerRuntime {
    docker: Docker,
}

fn docker_shell_entrypoint(setup: &str) -> Vec<String> {
    vec![
        "/bin/sh".into(),
        "-lc".into(),
        format!("{setup} && exec \"$@\""),
        "locus-agent".into(),
    ]
}

#[cfg(test)]
mod docker_entrypoint {
    use super::docker_shell_entrypoint;

    #[test]
    fn setup_execs_the_harness_command_passed_as_docker_cmd() {
        assert_eq!(
            docker_shell_entrypoint("prepare"),
            ["/bin/sh", "-lc", "prepare && exec \"$@\"", "locus-agent"]
        );
    }
}

impl DockerContainerRuntime {
    pub fn connect() -> Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_defaults().context("connect to Docker daemon")?,
        })
    }

    fn block_on<T: Send + 'static>(
        future: impl std::future::Future<Output = Result<T>> + Send + 'static,
    ) -> Result<T> {
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .expect("create Docker runtime")
                .block_on(future)
        })
        .join()
        .map_err(|_| anyhow::anyhow!("Docker runtime thread panicked"))?
    }
}

impl ContainerRuntime for DockerContainerRuntime {
    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition> {
        let docker = self.docker.clone();
        let image = image.to_owned();
        Self::block_on(async move {
            if docker.inspect_image(&image).await.is_ok() {
                return Ok(ImageDisposition::Reused);
            }

            let mut build = docker.build_image(
                BuildImageOptionsBuilder::default()
                    .t(&image)
                    .rm(true)
                    .build(),
                None,
                None,
            );
            while let Some(event) = build.next().await {
                let event = event.context("build agent image")?;
                if let Some(error) = event.error {
                    bail!("build agent image: {error}")
                }
            }
            docker
                .inspect_image(&image)
                .await
                .context("built agent image was not available")?;
            Ok(ImageDisposition::Built)
        })
    }

    fn start_container(&mut self, container: &ContainerLaunch) -> Result<()> {
        let docker = self.docker.clone();
        let launch = container.clone();
        Self::block_on(async move {
            let binds = launch
                .mounts
                .iter()
                .map(|mount| {
                    format!(
                        "{}:{}:{}",
                        mount.source,
                        mount.destination,
                        if mount.read_only { "ro" } else { "rw" }
                    )
                })
                .collect();
            let config = ContainerCreateBody {
                image: Some(launch.image),
                cmd: Some(launch.command),
                entrypoint: Some(docker_shell_entrypoint(&launch.entrypoint)),
                env: Some(launch.environment),
                tty: Some(true),
                open_stdin: Some(true),
                host_config: Some(HostConfig {
                    binds: Some(binds),
                    network_mode: Some(launch.network),
                    ..Default::default()
                }),
                ..Default::default()
            };
            docker
                .create_container(
                    Some(
                        CreateContainerOptionsBuilder::default()
                            .name(&launch.name)
                            .build(),
                    ),
                    config,
                )
                .await
                .context("create agent container")?;
            docker
                .start_container(&launch.name, None::<StartContainerOptions>)
                .await
                .context("start agent container")?;
            Ok(())
        })
    }

    fn attach_pty(
        &mut self,
        container: &str,
        attachment: PtyAttachment,
        stream: PtyStream,
    ) -> Result<()> {
        let docker = self.docker.clone();
        let container = container.to_owned();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().expect("create Docker runtime");
            runtime.block_on(async move {
                let mut attached = docker
                    .attach_container(
                        &container,
                        Some(
                            AttachContainerOptionsBuilder::default()
                                .stdin(attachment.tty)
                                .stdout(attachment.stdout)
                                .stderr(attachment.stderr)
                                .stream(true)
                                .logs(true)
                                .build(),
                        ),
                    )
                    .await?;
                while let Some(output) = attached.output.next().await {
                    let output: LogOutput = output?;
                    stream.write(output.as_ref());
                }
                Ok::<_, bollard::errors::Error>(())
            })
        });
        Ok(())
    }

    fn stop_container(&mut self, container: &str) -> Result<()> {
        let docker = self.docker.clone();
        let container = container.to_owned();
        Self::block_on(async move {
            docker
                .stop_container(&container, None::<StopContainerOptions>)
                .await
                .context("stop agent container")?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod streams_pty {
    use super::PtyStream;

    #[tokio::test]
    async fn delivers_pty_bytes_to_each_ui_subscriber() {
        let stream = PtyStream::new(2);
        let mut first_ui = stream.subscribe();
        let mut second_ui = stream.subscribe();

        stream.write(b"agent output");

        assert_eq!(first_ui.recv().await.unwrap(), b"agent output");
        assert_eq!(second_ui.recv().await.unwrap(), b"agent output");
    }
}
