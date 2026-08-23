//! The container supervisor: bollard over the Docker Engine API, and the PTY stream.

use std::{collections::HashMap, fs, path::Path};

use crate::bus::InProcessBus;
use anyhow::{bail, Context, Result};
use bollard::{
    body_full,
    container::LogOutput,
    models::{
        ContainerCreateBody, EndpointSettings, HostConfig, NetworkConnectRequest,
        NetworkCreateRequest, NetworkingConfig,
    },
    query_parameters::{
        AttachContainerOptionsBuilder, BuildImageOptionsBuilder, CreateContainerOptionsBuilder,
        InspectContainerOptionsBuilder, InspectNetworkOptionsBuilder,
        RemoveContainerOptionsBuilder, StartContainerOptions, StopContainerOptions,
    },
    Docker,
};
use futures::StreamExt;
use tokio::sync::broadcast;

use crate::sandbox::{
    forward_proxy::{
        policy_directory_is_empty, ForwardProxyLaunch, ForwardProxyPolicy, FORWARD_PROXY_ALIAS,
    },
    mounts::Mount,
    mounts::PtyAttachment,
};
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

    /// Prepare the project-private internal Docker network before an agent joins it.
    /// Default no-ops keep deterministic unit runtimes Docker-free.
    fn ensure_agent_network(&mut self, _network: &str) -> Result<()> {
        Ok(())
    }

    /// Start (or reuse) the Locus-built sidecar on the project's internal and egress networks.
    fn ensure_egress_proxy(&mut self, _proxy: &ForwardProxyLaunch) -> Result<()> {
        Ok(())
    }

    /// Revoke a policy and stop the sidecar when a project has no egress-capable runs left.
    fn release_egress_proxy(&mut self, _proxy: &ForwardProxyLaunch, _run_id: &str) -> Result<()> {
        Ok(())
    }
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

    fn ensure_agent_network(&mut self, network: &str) -> Result<()> {
        let docker = self.docker.clone();
        let network = network.to_owned();
        Self::block_on(async move { ensure_network(&docker, &network, true).await })
    }

    fn ensure_egress_proxy(&mut self, proxy: &ForwardProxyLaunch) -> Result<()> {
        let docker = self.docker.clone();
        let proxy = proxy.clone();
        Self::block_on(async move {
            ensure_network(&docker, &proxy.internal_network, true).await?;
            ensure_network(&docker, &proxy.egress_network, false).await?;
            ensure_forward_proxy_image(&docker, &proxy.image).await?;
            if docker
                .inspect_container(
                    &proxy.name,
                    Some(InspectContainerOptionsBuilder::default().build()),
                )
                .await
                .is_ok()
            {
                return Ok(());
            }

            let binds = vec![format!(
                "{}:/locus/policies:ro",
                proxy.policy_root.display()
            )];
            let mut endpoints = HashMap::new();
            endpoints.insert(
                proxy.internal_network.clone(),
                EndpointSettings {
                    aliases: Some(vec![FORWARD_PROXY_ALIAS.into()]),
                    ..Default::default()
                },
            );
            let config = ContainerCreateBody {
                image: Some(proxy.image.clone()),
                env: Some(vec!["LOCUS_POLICY_DIR=/locus/policies".into()]),
                host_config: Some(HostConfig {
                    binds: Some(binds),
                    // Only the sidecar receives the host-gateway mapping used by the
                    // credential broker; agents remain internal-network-only.
                    extra_hosts: Some(vec!["host.docker.internal:host-gateway".into()]),
                    network_mode: Some(proxy.internal_network.clone()),
                    readonly_rootfs: Some(true),
                    ..Default::default()
                }),
                networking_config: Some(NetworkingConfig {
                    endpoints_config: Some(endpoints),
                }),
                ..Default::default()
            };
            docker
                .create_container(
                    Some(
                        CreateContainerOptionsBuilder::default()
                            .name(&proxy.name)
                            .build(),
                    ),
                    config,
                )
                .await
                .context("create forwarding proxy sidecar")?;
            docker
                .start_container(&proxy.name, None::<StartContainerOptions>)
                .await
                .context("start forwarding proxy sidecar")?;
            docker
                .connect_network(
                    &proxy.egress_network,
                    NetworkConnectRequest {
                        container: Some(proxy.name.clone()),
                        endpoint_config: Some(EndpointSettings::default()),
                    },
                )
                .await
                .context("attach forwarding proxy to egress network")?;
            Ok(())
        })
    }

    fn release_egress_proxy(&mut self, proxy: &ForwardProxyLaunch, run_id: &str) -> Result<()> {
        ForwardProxyPolicy::remove_from(&proxy.policy_root, run_id)?;
        if !policy_directory_is_empty(&proxy.policy_root)? {
            return Ok(());
        }
        let docker = self.docker.clone();
        let proxy = proxy.clone();
        Self::block_on(async move {
            if docker
                .inspect_container(
                    &proxy.name,
                    Some(InspectContainerOptionsBuilder::default().build()),
                )
                .await
                .is_ok()
            {
                let _ = docker
                    .stop_container(&proxy.name, None::<StopContainerOptions>)
                    .await;
                docker
                    .remove_container(
                        &proxy.name,
                        Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                    )
                    .await
                    .context("remove forwarding proxy sidecar")?;
            }
            if docker
                .inspect_network(
                    &proxy.egress_network,
                    Some(InspectNetworkOptionsBuilder::default().build()),
                )
                .await
                .is_ok()
            {
                docker
                    .remove_network(&proxy.egress_network)
                    .await
                    .context("remove forwarding proxy egress network")?;
            }
            let _ = fs::remove_dir(&proxy.policy_root);
            Ok(())
        })
    }
}

async fn ensure_network(docker: &Docker, name: &str, internal: bool) -> Result<()> {
    if docker
        .inspect_network(name, Some(InspectNetworkOptionsBuilder::default().build()))
        .await
        .is_ok()
    {
        return Ok(());
    }
    docker
        .create_network(NetworkCreateRequest {
            name: name.into(),
            driver: Some("bridge".into()),
            internal: Some(internal),
            labels: Some(HashMap::from([("locus.managed".into(), "true".into())])),
            ..Default::default()
        })
        .await
        .with_context(|| format!("create Docker network `{name}`"))?;
    Ok(())
}

async fn ensure_forward_proxy_image(docker: &Docker, image: &str) -> Result<()> {
    if docker.inspect_image(image).await.is_ok() {
        return Ok(());
    }
    let context = vendored_proxy_context()?;
    let mut build = docker.build_image(
        BuildImageOptionsBuilder::default()
            .dockerfile("Dockerfile")
            .t(image)
            .rm(true)
            .build(),
        None,
        Some(body_full(context.into())),
    );
    while let Some(event) = build.next().await {
        let event = event.context("build Locus forwarding proxy image")?;
        if let Some(error) = event.error {
            bail!("build Locus forwarding proxy image: {error}")
        }
    }
    docker
        .inspect_image(image)
        .await
        .context("built forwarding proxy image was not available")?;
    Ok(())
}

fn vendored_proxy_context() -> Result<Vec<u8>> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("egress-proxy");
    let mut archive = tar::Builder::new(Vec::new());
    for name in ["Dockerfile", "main.rs"] {
        archive
            .append_path_with_name(source.join(name), name)
            .with_context(|| format!("add vendored proxy {name} to Docker build context"))?;
    }
    archive
        .finish()
        .context("finish forwarding proxy Docker context")?;
    archive
        .into_inner()
        .context("read forwarding proxy Docker context")
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
