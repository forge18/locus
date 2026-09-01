//! The container supervisor over the Docker Engine API.

use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{self, Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{bail, Context, Result};
use bollard::{
    body_full,
    container::LogOutput,
    exec::{CreateExecOptions, StartExecOptions, StartExecResults},
    models::{
        ContainerCreateBody, EndpointSettings, HostConfig, NetworkConnectRequest,
        NetworkCreateRequest, NetworkingConfig,
    },
    query_parameters::{
        BuildImageOptionsBuilder, CreateContainerOptionsBuilder, InspectContainerOptionsBuilder,
        InspectNetworkOptionsBuilder, RemoveContainerOptionsBuilder, StartContainerOptions,
        StopContainerOptions,
    },
    Docker,
};
use futures::StreamExt;
use tokio::{io::AsyncWriteExt, sync::mpsc as tokio_mpsc};

use crate::sandbox::{
    forward_proxy::{
        policy_directory_is_empty, ForwardProxyLaunch, ForwardProxyPolicy, FORWARD_PROXY_ALIAS,
    },
    mounts::Mount,
};
/// Whether the container runtime built the image or reused its existing cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDisposition {
    Built,
    Reused,
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerExecResult {
    pub status_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// The command and container identity for one run-owned debug adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugAdapterLaunch {
    pub container: String,
    pub command: Vec<String>,
    pub environment: Vec<String>,
    pub working_directory: Option<String>,
}

impl DebugAdapterLaunch {
    pub fn new(container: impl Into<String>, command: Vec<String>) -> Result<Self> {
        let container = container.into();
        if container.trim().is_empty() || command.is_empty() {
            bail!("debug adapter launch requires a container and command")
        }
        if command.iter().any(|part| part.trim().is_empty()) {
            bail!("debug adapter launch command must not contain empty arguments")
        }
        Ok(Self {
            container,
            command,
            environment: Vec::new(),
            working_directory: None,
        })
    }

    pub fn with_environment(mut self, environment: Vec<String>) -> Result<Self> {
        if environment.iter().any(|entry| entry.trim().is_empty()) {
            bail!("debug adapter environment must not contain empty entries")
        }
        self.environment = environment;
        Ok(self)
    }

    pub fn with_working_directory(mut self, directory: impl Into<String>) -> Result<Self> {
        let directory = directory.into();
        if directory.trim().is_empty() {
            bail!("debug adapter working directory must not be empty")
        }
        self.working_directory = Some(directory);
        Ok(self)
    }
}

/// The narrow container boundary required by run spawning.
///
/// The supplied container adapter owns image caching and container creation; this
/// supervisor owns their ordering and the run state transition.
pub trait ContainerRuntime: Send {
    fn backend(&self) -> super::backend::RuntimeBackend {
        super::backend::RuntimeBackend::Docker
    }

    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition>;
    fn prepare_container(&mut self, _container: &mut ContainerLaunch) -> Result<()> {
        Ok(())
    }
    fn start_container(&mut self, container: &ContainerLaunch) -> Result<()>;
    fn stop_container(&mut self, container: &str) -> Result<()>;

    fn attach_audit_sink(
        &mut self,
        _sink: Arc<dyn crate::sandbox::egress::AuditSink>,
    ) -> Result<()> {
        Ok(())
    }

    fn remove_container(&mut self, _container: &str) -> Result<()> {
        bail!("container removal is not supported by this runtime")
    }

    /// Run a non-interactive command in an existing agent workspace.
    fn exec(&mut self, _container: &str, _command: &[String]) -> Result<ContainerExecResult> {
        bail!("container command execution is not supported by this runtime")
    }

    fn container_is_alive(&mut self, _container: &str) -> Result<bool> {
        bail!("container state is not supported by this runtime")
    }

    /// Launch an adapter through the run container's stdio and return the owned DAP process.
    /// Lightweight runtimes may omit this capability, but production runtimes must not replace
    /// it with a host-side executable because the adapter must see the run's clone and image.
    fn launch_debug_adapter(
        &mut self,
        _launch: &DebugAdapterLaunch,
    ) -> Result<Box<dyn crate::runtime::dap::DebugAdapterProcess>> {
        bail!("container-backed debug adapters are not supported by this runtime")
    }

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

    /// Fresh verification is optional for lightweight runtimes. Implementations that support
    /// workflows must create a distinct container from the supplied request; the default keeps
    /// existing Docker-free test runtimes source-compatible.
    fn run_verify_container(
        &mut self,
        _request: &crate::services::workflow::VerifyContainerRequest,
    ) -> Result<crate::services::workflow::VerifyEvidence> {
        bail!("fresh-container verification is not supported by this runtime")
    }
}

impl<T> crate::services::workflow::VerifyContainerRunner for T
where
    T: ContainerRuntime,
{
    fn run_fresh_container(
        &mut self,
        request: &crate::services::workflow::VerifyContainerRequest,
    ) -> Result<crate::services::workflow::VerifyEvidence, crate::services::workflow::VerifyError>
    {
        self.run_verify_container(request)
            .map_err(|_| crate::services::workflow::VerifyError::RunnerUnavailable)
    }
}

/// Host-only Bollard adapter. Agents never receive the Docker client or its socket.
#[derive(Clone)]
pub struct DockerContainerRuntime {
    docker: Docker,
}

enum DebugAdapterInput {
    Bytes(Vec<u8>),
    Stop,
}

/// Blocking stdio facade over Docker's asynchronous exec attach stream.
///
/// DAP is a synchronous request/response protocol at the core boundary, while Bollard exposes
/// exec streams asynchronously. The bridge owns one Tokio task, forwards stdout to a bounded
/// blocking reader, and sends writes to the attached stdin. Dropping it closes the exec stdin;
/// the run container remains the final lifecycle owner.
struct DockerExecTransport {
    incoming: Mutex<mpsc::Receiver<io::Result<Vec<u8>>>>,
    pending: VecDeque<u8>,
    outgoing: tokio_mpsc::UnboundedSender<DebugAdapterInput>,
    stop: Arc<AtomicBool>,
}

impl DockerExecTransport {
    fn connect(docker: Docker, launch: DebugAdapterLaunch) -> Result<Self> {
        let (read_tx, read_rx) = mpsc::sync_channel::<io::Result<Vec<u8>>>(128);
        let (write_tx, write_rx) = tokio_mpsc::unbounded_channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<std::result::Result<(), String>>(1);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        std::thread::Builder::new()
            .name("locus-dap-exec".into())
            .spawn(move || {
                let result = match tokio::runtime::Runtime::new() {
                    Ok(runtime) => runtime.block_on(run_docker_debug_exec(
                        docker,
                        launch,
                        read_tx.clone(),
                        write_rx,
                        ready_tx.clone(),
                        thread_stop,
                    )),
                    Err(error) => Err(format!("create Docker DAP runtime: {error}")),
                };
                if let Err(error) = result {
                    let _ = ready_tx.send(Err(error.clone()));
                    let _ = read_tx.try_send(Err(io_error(error)));
                }
            })
            .context("start Docker DAP bridge thread")?;
        match ready_rx
            .recv_timeout(Duration::from_secs(10))
            .context("wait for Docker debug adapter")?
        {
            Ok(()) => Ok(Self {
                incoming: Mutex::new(read_rx),
                pending: VecDeque::new(),
                outgoing: write_tx,
                stop,
            }),
            Err(error) => Err(anyhow::anyhow!(error)),
        }
    }
}

impl Read for DockerExecTransport {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        while self.pending.is_empty() {
            let message = self
                .incoming
                .lock()
                .map_err(|_| io_error("Docker DAP reader lock is poisoned"))?
                .recv()
                .map_err(|_| io_error("Docker debug adapter closed its output"))?;
            let bytes = message?;
            self.pending.extend(bytes);
        }
        let count = buffer.len().min(self.pending.len());
        for slot in &mut buffer[..count] {
            let Some(byte) = self.pending.pop_front() else {
                return Err(io_error("Docker DAP buffer underflow"));
            };
            *slot = byte;
        }
        Ok(count)
    }
}

impl Write for DockerExecTransport {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.outgoing
            .send(DebugAdapterInput::Bytes(buffer.to_vec()))
            .map_err(|_| io_error("Docker debug adapter stdin is closed"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for DockerExecTransport {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.outgoing.send(DebugAdapterInput::Stop);
    }
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

async fn forward_debug_output(
    sender: &mpsc::SyncSender<io::Result<Vec<u8>>>,
    mut message: io::Result<Vec<u8>>,
    stop: &AtomicBool,
) -> bool {
    loop {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        match sender.try_send(message) {
            Ok(()) => return true,
            Err(mpsc::TrySendError::Full(next)) => {
                message = next;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(mpsc::TrySendError::Disconnected(_)) => return false,
        }
    }
}

async fn run_docker_debug_exec(
    docker: Docker,
    launch: DebugAdapterLaunch,
    read_tx: mpsc::SyncSender<io::Result<Vec<u8>>>,
    mut write_rx: tokio_mpsc::UnboundedReceiver<DebugAdapterInput>,
    ready_tx: mpsc::SyncSender<std::result::Result<(), String>>,
    stop: Arc<AtomicBool>,
) -> std::result::Result<(), String> {
    let exec = docker
        .create_exec(
            &launch.container,
            CreateExecOptions {
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(false),
                env: (!launch.environment.is_empty()).then_some(launch.environment),
                cmd: Some(launch.command),
                working_dir: launch.working_directory,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| format!("create debug adapter exec: {error}"))?;
    let attached = docker
        .start_exec(
            &exec.id,
            Some(StartExecOptions {
                detach: false,
                tty: false,
                output_capacity: Some(crate::runtime::dap::MAX_DAP_FRAME_BYTES),
            }),
        )
        .await
        .map_err(|error| format!("start debug adapter exec: {error}"))?;
    let (mut output, mut input) = match attached {
        StartExecResults::Attached { output, input } => (output, input),
        StartExecResults::Detached => {
            let error = "debug adapter exec unexpectedly detached".to_owned();
            let _ = ready_tx.send(Err(error.clone()));
            return Err(error);
        }
    };
    let _ = ready_tx.send(Ok(()));
    loop {
        tokio::select! {
            item = output.next() => {
                match item {
                    Some(Ok(output)) => {
                        if matches!(output, LogOutput::StdOut { .. } | LogOutput::Console { .. }) {
                            let bytes = output.into_bytes().to_vec();
                            if !bytes.is_empty()
                                && !forward_debug_output(&read_tx, Ok(bytes), &stop).await
                            {
                                return Ok(());
                            }
                        }
                    }
                    Some(Err(error)) => {
                        return Err(format!("read debug adapter output: {error}"));
                    }
                    None => return Err("debug adapter output closed".to_owned()),
                }
            }
            command = write_rx.recv() => {
                match command {
                    Some(DebugAdapterInput::Bytes(bytes)) => {
                        input.write_all(&bytes).await.map_err(|error| format!("write debug adapter input: {error}"))?;
                        input.flush().await.map_err(|error| format!("flush debug adapter input: {error}"))?;
                    }
                    Some(DebugAdapterInput::Stop) | None => return Ok(()),
                }
            }
        }
    }
}

fn docker_shell_entrypoint(setup: &str) -> Vec<String> {
    vec![
        "/bin/sh".into(),
        "-lc".into(),
        format!("{setup} && exec \"$@\""),
        "locus-agent".into(),
    ]
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
                tty: Some(false),
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

    fn remove_container(&mut self, container: &str) -> Result<()> {
        let docker = self.docker.clone();
        let container = container.to_owned();
        Self::block_on(async move {
            docker
                .remove_container(
                    &container,
                    Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                )
                .await
                .context("remove agent container")?;
            Ok(())
        })
    }

    fn exec(&mut self, container: &str, command: &[String]) -> Result<ContainerExecResult> {
        if command.is_empty() {
            bail!("container command must not be empty")
        }
        let docker = self.docker.clone();
        let container = container.to_owned();
        let command = command.to_vec();
        Self::block_on(async move {
            let exec = docker
                .create_exec(
                    &container,
                    CreateExecOptions {
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        cmd: Some(command),
                        working_dir: Some("/workspace".into()),
                        ..Default::default()
                    },
                )
                .await
                .context("create container command")?;
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            match docker
                .start_exec(
                    &exec.id,
                    Some(StartExecOptions {
                        detach: false,
                        tty: false,
                        output_capacity: None,
                    }),
                )
                .await
                .context("start container command")?
            {
                StartExecResults::Attached { mut output, .. } => {
                    while let Some(message) = output.next().await {
                        match message.context("read container command output")? {
                            LogOutput::StdOut { message } => stdout.extend(message),
                            LogOutput::StdErr { message } => stderr.extend(message),
                            _ => {}
                        }
                    }
                }
                StartExecResults::Detached => bail!("container command unexpectedly detached"),
            }
            let status_code = docker
                .inspect_exec(&exec.id)
                .await
                .context("inspect container command")?
                .exit_code
                .map(i32::try_from)
                .transpose()
                .context("container command exit code exceeds i32")?
                .unwrap_or(-1);
            Ok(ContainerExecResult {
                status_code,
                stdout,
                stderr,
            })
        })
    }

    fn container_is_alive(&mut self, container: &str) -> Result<bool> {
        <Self as crate::runtime::boot::BootRuntime>::container_is_alive(self, container)
    }

    fn launch_debug_adapter(
        &mut self,
        launch: &DebugAdapterLaunch,
    ) -> Result<Box<dyn crate::runtime::dap::DebugAdapterProcess>> {
        let transport = DockerExecTransport::connect(self.docker.clone(), launch.clone())?;
        Ok(Box::new(crate::runtime::dap::DapClientProcess::new(
            transport,
        )))
    }

    fn ensure_agent_network(&mut self, network: &str) -> Result<()> {
        let docker = self.docker.clone();
        let network = network.to_owned();
        Self::block_on(async move { ensure_network(&docker, &network, true).await })
    }

    fn run_verify_container(
        &mut self,
        request: &crate::services::workflow::VerifyContainerRequest,
    ) -> Result<crate::services::workflow::VerifyEvidence> {
        let docker = self.docker.clone();
        let request = request.clone();
        Self::block_on(async move { run_verify_container(&docker, &request).await })
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

impl crate::runtime::boot::BootRuntime for DockerContainerRuntime {
    fn container_is_alive(&mut self, container: &str) -> Result<bool> {
        let docker = self.docker.clone();
        let container = container.to_owned();
        Self::block_on(async move {
            match docker
                .inspect_container(
                    &container,
                    Some(InspectContainerOptionsBuilder::default().build()),
                )
                .await
            {
                Ok(inspected) => Ok(inspected
                    .state
                    .and_then(|state| state.running)
                    .unwrap_or(false)),
                Err(error)
                    if error.to_string().contains("No such container")
                        || error.to_string().contains("No such object") =>
                {
                    Ok(false)
                }
                Err(error) => Err(error).context("inspect Docker agent container state"),
            }
        })
    }

    fn reattach_agent(&mut self, container: &str) -> Result<()> {
        if !<Self as ContainerRuntime>::container_is_alive(self, container)? {
            bail!("agent container `{container}` is not running")
        }
        Ok(())
    }
}

async fn run_verify_container(
    docker: &Docker,
    request: &crate::services::workflow::VerifyContainerRequest,
) -> Result<crate::services::workflow::VerifyEvidence> {
    let config = ContainerCreateBody {
        // The verifier is deliberately a second container from the exact image selected for the
        // agent. It receives no mounts, so success cannot depend on the agent container's dirtied
        // filesystem.
        image: Some(request.image.clone()),
        cmd: Some(vec![request.command_line()]),
        entrypoint: Some(vec!["/bin/sh".into(), "-lc".into()]),
        tty: Some(false),
        open_stdin: Some(false),
        ..Default::default()
    };
    let created = docker
        .create_container(
            Some(
                CreateContainerOptionsBuilder::default()
                    .name(&request.container_name)
                    .build(),
            ),
            config,
        )
        .await
        .context("create fresh verification container")?;
    let result = async {
        docker
            .start_container(&request.container_name, None::<StartContainerOptions>)
            .await
            .context("start fresh verification container")?;
        let mut wait = docker.wait_container(
            &request.container_name,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );
        let exit_code = match wait.next().await {
            Some(Ok(response)) => response.status_code as i32,
            Some(Err(bollard::errors::Error::DockerContainerWaitError { code, .. })) => code as i32,
            Some(Err(error)) => return Err(error).context("wait for fresh verification container"),
            None => bail!("fresh verification container exited without a status"),
        };
        let mut logs = docker.logs(
            &request.container_name,
            Some(
                bollard::query_parameters::LogsOptionsBuilder::default()
                    .stdout(true)
                    .stderr(true)
                    .follow(false)
                    .timestamps(false)
                    .tail("all")
                    .build(),
            ),
        );
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        while let Some(output) = logs.next().await {
            match output.context("read fresh verification output")? {
                LogOutput::StdOut { message } | LogOutput::Console { message } => {
                    stdout.extend_from_slice(&message)
                }
                LogOutput::StdErr { message } => stderr.extend_from_slice(&message),
                LogOutput::StdIn { .. } => {}
            }
        }
        Ok(crate::services::workflow::VerifyEvidence {
            exit_code,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            passed: exit_code == 0,
            command: request.command.clone(),
            container_id: created.id.clone(),
            verify_node_id: request.verify_node_id.clone(),
        })
    }
    .await;
    let removal = docker
        .remove_container(
            &request.container_name,
            Some(RemoveContainerOptionsBuilder::default().force(true).build()),
        )
        .await
        .context("remove fresh verification container");
    match (result, removal) {
        (Ok(evidence), Ok(())) => Ok(evidence),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(_removal_error)) => Err(error),
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
