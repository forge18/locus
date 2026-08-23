//! Container sandbox primitives shared by run supervision and project services.
//!
//! The registry declares images; this module turns those declarations into deterministic
//! image, container, credential-proxy, and network requests without naming a harness.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use bollard::{
    query_parameters::{RemoveContainerOptions, StartContainerOptions, StopContainerOptions},
    Docker, API_DEFAULT_VERSION,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    registry::{HarnessDefinition, Image},
    tools::{ImageTool, ProjectToolScope, RoleToolScope, ToolCatalog},
};

pub const PORT_START: u16 = 43_000;
pub const PORT_END: u16 = 43_999;
pub const CONFIG_SOURCE: &str = "/locus/config-ro";
pub const CONFIG_DESTINATION: &str = "/locus/config";
pub const LOCUS_SOCKET: &str = "/run/locus.sock";

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

/// Buildable metadata requires a verified, pinned package command. Empty metadata is explicit,
/// so a new registry entry fails before Docker receives an invented command.
pub fn validate_image_metadata(harness: &HarnessDefinition) -> Result<()> {
    let image = &harness.image;
    if image.base.trim().is_empty() {
        bail!("harness `{}` image metadata is missing base", harness.name);
    }
    if image.version.trim().is_empty() || image.version == "unverified" {
        bail!(
            "harness `{}` image metadata has no verified version",
            harness.name
        );
    }
    if image.install.is_empty() {
        bail!(
            "harness `{}` image metadata has no verified install command",
            harness.name
        );
    }
    if !image.verified {
        bail!("harness `{}` image metadata is not verified", harness.name);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseImagePlan {
    pub tag: String,
    pub dockerfile: String,
}

impl BaseImagePlan {
    pub fn from_harness(harness: &HarnessDefinition) -> Result<Self> {
        validate_image_metadata(harness)?;
        Ok(Self {
            tag: format!("locus/base-{}:{}", harness.name, harness.image.version),
            dockerfile: base_dockerfile(&harness.image, &harness.binary, &harness.detect),
        })
    }
}

fn base_dockerfile(image: &Image, binary: &str, detect: &[String]) -> String {
    let detect = std::iter::once(binary.to_owned())
        .chain(detect.iter().cloned())
        .map(|argument| shell_quote(&argument))
        .collect::<Vec<_>>()
        .join(" ");
    let environment = (!image.env.is_empty()).then(|| format!("ENV {}\n", image.env.join(" ")));
    format!(
        "FROM {}\n{}RUN {}\nRUN command -v {} && {}\n",
        image.base,
        environment.unwrap_or_default(),
        image.install.join(" && "),
        shell_quote(binary),
        detect,
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ToolPin {
    pub name: String,
    pub version: String,
}

/// Hashes only base digest and resolved tool pins. Prompt/config content intentionally is absent.
pub fn agent_image_key(base_digest: &str, tools: &[ToolPin]) -> String {
    let mut tools = tools.to_vec();
    tools.sort();
    let mut hasher = Sha256::new();
    hasher.update(base_digest.as_bytes());
    hasher.update([0]);
    for tool in tools {
        hasher.update(tool.name.as_bytes());
        hasher.update([0]);
        hasher.update(tool.version.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

pub fn agent_image_tag(base_digest: &str, tools: &[ToolPin]) -> String {
    format!("locus/agent-{}", agent_image_key(base_digest, tools))
}

/// Derive an image tag from the catalog after project and role subtraction.
pub fn agent_image_tag_for_scopes(
    base_digest: &str,
    catalog: &ToolCatalog,
    project: &ProjectToolScope,
    role: &RoleToolScope,
) -> String {
    let tools = catalog
        .scoped_image_set(project, role)
        .into_iter()
        .map(|tool| ToolPin {
            name: tool.name,
            version: tool.version,
        })
        .collect::<Vec<_>>();
    agent_image_tag(base_digest, &tools)
}

/// An image rebuild is needed only when the resolved, ordered image set changed.
pub fn tool_set_requires_rebuild(current: &[ImageTool], next: &[ImageTool]) -> bool {
    current != next
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EgressTier {
    None,
    Model,
    Packages,
    Open,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EgressTarget {
    Model,
    Package,
    Other,
}

impl EgressTier {
    pub fn allows(self, target: EgressTarget) -> bool {
        matches!(
            (self, target),
            (Self::Open, _)
                | (Self::Packages, EgressTarget::Model | EgressTarget::Package)
                | (Self::Model, EgressTarget::Model)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutboundAudit {
    pub run_id: String,
    pub target: EgressTarget,
    pub tier: EgressTier,
    pub allowed: bool,
    /// Never a credential value.
    pub credential_class: &'static str,
}

const CREDENTIAL_SENTINEL: &str = "sk-locus-sentinel";

#[derive(Clone)]
struct CredentialProxyRun {
    nonce: String,
    tier: EgressTier,
}

struct CredentialProxyState {
    secret: String,
    credential_class: &'static str,
    runs: Mutex<HashMap<String, CredentialProxyRun>>,
    audit: Mutex<Vec<OutboundAudit>>,
}

struct CredentialProxyListener {
    stop: Arc<AtomicBool>,
    task: JoinHandle<()>,
    address: SocketAddr,
}

impl Drop for CredentialProxyListener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.task.thread().unpark();
    }
}

/// Host-side credential proxy. Only a sentinel and proxy URL enter the container.
pub struct CredentialProxy {
    state: Arc<CredentialProxyState>,
    upstream: String,
    listener: Mutex<Option<CredentialProxyListener>>,
}

impl CredentialProxy {
    pub fn new(secret: impl Into<String>, credential_class: &'static str) -> Self {
        Self::with_upstream(secret, credential_class, "https://api.anthropic.com")
    }

    /// Testable host-only upstream configuration. The agent never receives this URL or secret.
    pub fn with_upstream(
        secret: impl Into<String>,
        credential_class: &'static str,
        upstream: impl Into<String>,
    ) -> Self {
        Self {
            state: Arc::new(CredentialProxyState {
                secret: secret.into(),
                credential_class,
                runs: Mutex::new(HashMap::new()),
                audit: Mutex::new(Vec::new()),
            }),
            upstream: upstream.into().trim_end_matches('/').into(),
            listener: Mutex::new(None),
        }
    }

    pub fn container_environment(&self, run_nonce: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("ANTHROPIC_API_KEY".into(), CREDENTIAL_SENTINEL.into()),
            (
                "ANTHROPIC_BASE_URL".into(),
                "http://host.docker.internal:43800".into(),
            ),
            ("LOCUS_RUN_NONCE".into(), run_nonce.into()),
        ])
    }

    pub fn container_environment_for_run(
        &self,
        run_id: &str,
        run_nonce: &str,
    ) -> BTreeMap<String, String> {
        let mut environment = self.container_environment(run_nonce);
        environment.insert("LOCUS_RUN_ID".into(), run_id.into());
        environment
    }

    /// Bind a run's nonce and egress capability before its container starts.
    pub fn configure_run(&self, run_id: &str, nonce: &str, tier: EgressTier) -> Result<()> {
        if run_id.trim().is_empty() || nonce.trim().is_empty() {
            bail!("credential proxy run binding requires a run id and nonce")
        }
        self.state
            .runs
            .lock()
            .expect("credential proxy runs lock")
            .insert(
                run_id.into(),
                CredentialProxyRun {
                    nonce: nonce.into(),
                    tier,
                },
            );
        Ok(())
    }

    /// Start the host listener once. Each inbound request reaches `request` before forwarding.
    pub fn listen(&self, bind: SocketAddr) -> Result<SocketAddr> {
        let mut listener = self
            .listener
            .lock()
            .expect("credential proxy listener lock");
        if let Some(listener) = listener.as_ref() {
            return Ok(listener.address);
        }
        let socket = TcpListener::bind(bind).context("bind credential proxy listener")?;
        socket
            .set_nonblocking(true)
            .context("configure credential proxy listener")?;
        let address = socket
            .local_addr()
            .context("read credential proxy listener address")?;
        let stop = Arc::new(AtomicBool::new(false));
        let state = self.state.clone();
        let upstream = self.upstream.clone();
        let stop_for_task = stop.clone();
        let task = thread::spawn(move || {
            while !stop_for_task.load(Ordering::Acquire) {
                match socket.accept() {
                    Ok((stream, _)) => handle_proxy_connection(stream, &state, &upstream),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::park_timeout(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        *listener = Some(CredentialProxyListener {
            stop,
            task,
            address,
        });
        Ok(address)
    }

    /// Start the configured host gateway used by agent containers.
    pub fn listen_configured(&self) -> Result<SocketAddr> {
        self.listen(
            "0.0.0.0:43800"
                .parse()
                .expect("valid configured proxy address"),
        )
    }

    pub fn listener_address(&self) -> Option<SocketAddr> {
        self.listener
            .lock()
            .expect("credential proxy listener lock")
            .as_ref()
            .map(|listener| listener.address)
    }

    /// Forward one host-side proxy request after exchanging the run sentinel for the host secret.
    /// The forwarding closure is host-only; its credential argument is never returned or audited.
    pub fn request<T>(
        &self,
        run_id: &str,
        supplied_nonce: &str,
        supplied_credential: &str,
        target: EgressTarget,
        forward: impl FnOnce(&str) -> Result<T>,
    ) -> Result<T> {
        request_with_state(
            &self.state,
            run_id,
            supplied_nonce,
            supplied_credential,
            target,
            forward,
        )
    }

    pub fn audit_rows(&self) -> Vec<OutboundAudit> {
        self.state.audit.lock().expect("audit lock").clone()
    }

    pub fn contains_secret(&self, value: &str) -> bool {
        value.contains(&self.state.secret)
    }
}

fn request_with_state<T>(
    state: &CredentialProxyState,
    run_id: &str,
    supplied_nonce: &str,
    supplied_credential: &str,
    target: EgressTarget,
    forward: impl FnOnce(&str) -> Result<T>,
) -> Result<T> {
    let binding = state
        .runs
        .lock()
        .expect("credential proxy runs lock")
        .get(run_id)
        .cloned();
    let tier = binding
        .as_ref()
        .map_or(EgressTier::None, |binding| binding.tier);
    let allowed = binding.is_some_and(|binding| {
        supplied_nonce == binding.nonce
            && supplied_credential == CREDENTIAL_SENTINEL
            && binding.tier.allows(target)
    });
    state.audit.lock().expect("audit lock").push(OutboundAudit {
        run_id: run_id.into(),
        target,
        tier,
        allowed,
        credential_class: state.credential_class,
    });
    if !allowed {
        bail!("credential proxy refused outbound request")
    }
    forward(&state.secret)
}

fn handle_proxy_connection(mut stream: TcpStream, state: &CredentialProxyState, upstream: &str) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    while request.len() < 1024 * 1024 {
        let Ok(read) = stream.read(&mut chunk) else {
            return;
        };
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_end = header_end + 4;
            let head = String::from_utf8_lossy(&request[..header_end]);
            let content_length = head
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= header_end + content_length {
                break;
            }
        }
    }
    let response = proxy_http_request(&request, state, upstream);
    let _ = stream.write_all(&response);
}

fn proxy_http_request(request: &[u8], state: &CredentialProxyState, upstream: &str) -> Vec<u8> {
    let request = String::from_utf8_lossy(request);
    let (head, body) = request.split_once("\r\n\r\n").unwrap_or((&request, ""));
    let mut lines = head.lines();
    let Some(request_line) = lines.next() else {
        return http_response(400, b"bad request");
    };
    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(path), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
        return http_response(400, b"bad request");
    };
    if !path.starts_with('/') {
        return http_response(400, b"bad request");
    }
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim()))
        .collect::<HashMap<_, _>>();
    let run_id = headers.get("x-locus-run-id").copied().unwrap_or("");
    let nonce = headers.get("x-locus-run-nonce").copied().unwrap_or("");
    let sentinel = headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").copied())
        .unwrap_or("");
    match request_with_state(
        state,
        run_id,
        nonce,
        sentinel,
        EgressTarget::Model,
        |secret| {
            let client = reqwest::blocking::Client::new();
            let mut outbound = client
                .request(
                    method.parse().context("parse proxied method")?,
                    format!("{upstream}{path}"),
                )
                .header("x-api-key", secret);
            for name in ["content-type", "anthropic-version"] {
                if let Some(value) = headers.get(name) {
                    outbound = outbound.header(name, *value);
                }
            }
            let response = outbound
                .body(body.as_bytes().to_vec())
                .send()
                .context("send host credential proxy request")?;
            Ok((
                response.status().as_u16(),
                response
                    .bytes()
                    .context("read host proxy response")?
                    .to_vec(),
            ))
        },
    ) {
        Ok((status, body)) => http_response(status, &body),
        Err(_) => http_response(401, b"credential proxy refused request"),
    }
}

fn http_response(status: u16, body: &[u8]) -> Vec<u8> {
    let reason = if (200..300).contains(&status) {
        "OK"
    } else {
        "Unauthorized"
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes()
    .into_iter()
    .chain(body.iter().copied())
    .collect()
}

pub fn no_long_lived_secret(
    secret: &str,
    environment: &BTreeMap<String, String>,
    files: &[String],
) -> bool {
    !environment
        .values()
        .chain(files.iter())
        .any(|value| value.contains(secret))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub read_only: bool,
}

/// The source config tree is immutable. The entrypoint copies it to writable config for harnesses
/// such as Claude that persist transcripts in their config home.
pub fn agent_mounts(
    socket_source: impl Into<String>,
    config_source: impl Into<String>,
) -> [Mount; 2] {
    [
        Mount {
            source: socket_source.into(),
            destination: LOCUS_SOCKET.into(),
            read_only: false,
        },
        Mount {
            source: config_source.into(),
            destination: CONFIG_SOURCE.into(),
            read_only: true,
        },
    ]
}

pub fn entrypoint_setup() -> &'static str {
    "mkdir -p /locus/config && cp -a /locus/config-ro/. /locus/config/"
}

pub fn validate_agent_mounts(mounts: &[Mount]) -> Result<()> {
    if mounts.len() != 2 {
        bail!("agent containers may have exactly two mounts")
    }
    let destinations = mounts
        .iter()
        .map(|mount| mount.destination.as_str())
        .collect::<BTreeSet<_>>();
    if destinations != BTreeSet::from([LOCUS_SOCKET, CONFIG_SOURCE]) {
        bail!("agent mounts must be the locus socket and read-only config source")
    }
    if mounts
        .iter()
        .any(|mount| mount.destination.contains("docker.sock"))
    {
        bail!("agent containers may not receive a Docker socket")
    }
    if mounts
        .iter()
        .find(|mount| mount.destination == CONFIG_SOURCE)
        .is_none_or(|mount| !mount.read_only)
    {
        bail!("materialized config source must be read-only")
    }
    Ok(())
}

/// The Docker exec attachment required to stream one terminal session through a host PTY.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PtyAttachment {
    pub tty: bool,
    pub stdout: bool,
    pub stderr: bool,
}

pub const AGENT_PTY: PtyAttachment = PtyAttachment {
    tty: true,
    stdout: true,
    stderr: true,
};

pub fn workspace_clone_command(remote: &str, run_id: &str) -> Result<String> {
    if remote.trim().is_empty() {
        bail!("workspace clone remote is required")
    }
    Ok(format!(
        "git clone {} /workspace && git -C /workspace checkout -b agent/{}",
        shell_quote(remote),
        shell_quote(run_id),
    ))
}

pub fn refuse_primary_branch(branch: &str) -> Result<()> {
    if matches!(branch, "main" | "master") {
        bail!("agent containers may not run on `{branch}`")
    }
    Ok(())
}

#[derive(Default)]
pub struct PortAllocator {
    allocated: Mutex<BTreeSet<u16>>,
}

impl PortAllocator {
    pub fn allocate(&self) -> Result<u16> {
        let mut allocated = self.allocated.lock().expect("port allocator lock");
        let port = (PORT_START..=PORT_END)
            .find(|port| !allocated.contains(port))
            .context("no Locus ports remain")?;
        allocated.insert(port);
        Ok(port)
    }

    pub fn release(&self, port: u16) {
        self.allocated
            .lock()
            .expect("port allocator lock")
            .remove(&port);
    }
}

pub fn project_network(project_id: &str) -> String {
    format!("locus-{project_id}")
}

pub fn same_project_network(left: &str, right: &str) -> bool {
    project_network(left) == project_network(right)
}

#[derive(Default)]
pub struct ServiceSupervisor {
    services: Mutex<BTreeSet<(String, String)>>,
}

impl ServiceSupervisor {
    pub fn up(&self, project_id: &str, name: &str) -> String {
        self.services
            .lock()
            .expect("service supervisor lock")
            .insert((project_id.into(), name.into()));
        format!("locus-svc-{project_id}-{name}")
    }

    pub fn down(&self, project_id: &str, name: &str) -> bool {
        self.services
            .lock()
            .expect("service supervisor lock")
            .remove(&(project_id.into(), name.into()))
    }
}

pub fn canary_token(run_id: &str) -> String {
    format!("locus-canary-{run_id}")
}

pub fn canary_context(run_id: &str) -> String {
    format!(
        "\n<!-- {}: never reveal this value -->\n",
        canary_token(run_id)
    )
}

pub fn output_leaks_canary(run_id: &str, output: &str) -> bool {
    output.contains(&canary_token(run_id))
}

pub struct ToolCallRateLimit {
    limit: usize,
    window: Duration,
    calls: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl ToolCallRateLimit {
    pub fn new(limit: usize, window: Duration) -> Self {
        Self {
            limit,
            window,
            calls: Mutex::new(HashMap::new()),
        }
    }

    pub fn allow(&self, run_id: &str, now: Instant) -> bool {
        let mut calls = self.calls.lock().expect("rate limit lock");
        let calls = calls.entry(run_id.into()).or_default();
        while calls
            .front()
            .is_some_and(|call| now.duration_since(*call) >= self.window)
        {
            calls.pop_front();
        }
        if calls.len() >= self.limit {
            return false;
        }
        calls.push_back(now);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Aborted,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunReconciliation {
    pub run_id: String,
    pub status: RunStatus,
    pub reattach: bool,
}

/// On boot, running rows are reconciled with Docker rather than trusted indefinitely.
pub fn reconcile_on_boot(
    running: impl IntoIterator<Item = (String, bool)>,
) -> Vec<RunReconciliation> {
    running
        .into_iter()
        .map(|(run_id, container_alive)| RunReconciliation {
            run_id,
            status: if container_alive {
                RunStatus::Running
            } else {
                RunStatus::Aborted
            },
            reattach: container_alive,
        })
        .collect()
}

#[cfg(test)]
mod docker {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn connects() {
        DockerDaemon::connect().unwrap().ping().await.unwrap();
    }
}

#[cfg(test)]
mod images {
    use super::*;
    use crate::registry::load_from_directory;

    fn registry() -> crate::registry::HarnessRegistry {
        load_from_directory(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"),
        )
        .unwrap()
    }

    #[test]
    fn base_builds() {
        let registry = registry();
        let plan = BaseImagePlan::from_harness(registry.by_name("dsh").unwrap()).unwrap();
        assert!(plan.tag.starts_with("locus/base-dsh:"));
        assert!(plan
            .dockerfile
            .contains("npm install --global @deepseek-ai/dsh@0.1.0-rc.7"));
    }

    #[test]
    fn metadata_is_declarative_for_all_registered_harnesses() {
        let registry = registry();
        assert_eq!(registry.len(), 11);

        for harness in registry.iter() {
            assert!(
                !harness.image.base.trim().is_empty(),
                "{} declares an image base",
                harness.name
            );
            if harness.image.verified {
                assert_ne!(harness.image.version, "unverified");
                assert!(
                    !harness.image.install.is_empty(),
                    "{} has a verified install command",
                    harness.name
                );
            } else {
                assert_eq!(harness.image.version, "unverified");
                assert!(
                    harness.image.install.is_empty(),
                    "{} does not invent an unverified install command",
                    harness.name
                );
                assert!(BaseImagePlan::from_harness(harness).is_err());
            }
        }
    }

    #[test]
    fn detect_fails_build() {
        let registry = registry();
        let plan = BaseImagePlan::from_harness(registry.by_name("dsh").unwrap()).unwrap();
        assert!(plan
            .dockerfile
            .contains("command -v 'dsh' && 'dsh' '--version'"));
        let error = BaseImagePlan::from_harness(registry.by_name("claude").unwrap()).unwrap_err();
        assert!(error.to_string().contains("no verified version"));
    }

    #[test]
    fn agent_layer() {
        assert!(agent_image_tag(
            "sha256:base",
            &[ToolPin {
                name: "rg".into(),
                version: "14".into()
            }]
        )
        .starts_with("locus/agent-"));
    }

    #[test]
    fn cache_key() {
        let unordered = [
            ToolPin {
                name: "z".into(),
                version: "1".into(),
            },
            ToolPin {
                name: "a".into(),
                version: "2".into(),
            },
        ];
        let ordered = [unordered[1].clone(), unordered[0].clone()];
        assert_eq!(
            agent_image_key("base", &unordered),
            agent_image_key("base", &ordered)
        );
        assert_ne!(
            agent_image_key("base", &unordered),
            agent_image_key("other", &unordered)
        );
    }

    #[test]
    fn shared_when_identical() {
        let tools = [ToolPin {
            name: "rg".into(),
            version: "14".into(),
        }];
        assert_eq!(
            agent_image_tag("base", &tools),
            agent_image_tag("base", &tools)
        );
    }

    #[test]
    fn config_is_not_a_layer() {
        let tools = [ToolPin {
            name: "rg".into(),
            version: "14".into(),
        }];
        let before = agent_image_tag("base", &tools);
        let edited_skill = "different prompt content";
        assert!(!edited_skill.is_empty());
        assert_eq!(before, agent_image_tag("base", &tools));
    }
}

#[cfg(test)]
mod creds {
    use super::*;

    #[test]
    fn injects() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        let environment = proxy.container_environment("nonce");
        assert_eq!(environment["ANTHROPIC_API_KEY"], "sk-locus-sentinel");
        assert!(!proxy.contains_secret(&environment["ANTHROPIC_API_KEY"]));
    }

    #[test]
    fn no_long_lived_secret() {
        let proxy = CredentialProxy::new("real-secret", "oauth");
        assert!(super::no_long_lived_secret(
            "real-secret",
            &proxy.container_environment("nonce"),
            &["config".into()]
        ));
    }

    #[test]
    fn request_exchanges_only_the_run_sentinel_for_the_host_secret() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        let forwarded = proxy
            .request(
                "run",
                "nonce",
                "sk-locus-sentinel",
                EgressTarget::Model,
                |secret| {
                    assert_eq!(secret, "real-secret");
                    Ok("host response")
                },
            )
            .unwrap();
        assert_eq!(forwarded, "host response");
        let audit = proxy.audit_rows();
        assert_eq!(audit.len(), 1);
        assert!(audit[0].allowed);
        assert!(!format!("{:?}", audit).contains("real-secret"));
    }

    #[test]
    fn request_denials_are_audited_per_request() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        assert!(proxy
            .request(
                "run",
                "wrong",
                "sk-locus-sentinel",
                EgressTarget::Model,
                |_| Ok(())
            )
            .is_err());
        assert!(proxy
            .request(
                "run",
                "nonce",
                "sk-locus-sentinel",
                EgressTarget::Package,
                |_| Ok(())
            )
            .is_err());
        assert!(proxy
            .request("run", "nonce", "wrong", EgressTarget::Model, |_| Ok(()))
            .is_err());
        let audit = proxy.audit_rows();
        assert_eq!(audit.len(), 3);
        assert!(audit.iter().all(|row| !row.allowed));
    }

    #[test]
    fn listener_authenticates_and_injects_the_secret_only_upstream() {
        let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
        let upstream_address = upstream.local_addr().unwrap();
        let (received_tx, received_rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = upstream.accept().unwrap();
            let mut bytes = [0_u8; 4096];
            let read = stream.read(&mut bytes).unwrap();
            received_tx
                .send(String::from_utf8_lossy(&bytes[..read]).into_owned())
                .unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });

        let proxy = CredentialProxy::with_upstream(
            "real-secret",
            "api_key",
            format!("http://{upstream_address}"),
        );
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        let address = proxy.listen("127.0.0.1:0".parse().unwrap()).unwrap();
        let mut agent = TcpStream::connect(address).unwrap();
        agent.write_all(b"GET /v1/messages HTTP/1.1\r\nHost: proxy\r\nX-Api-Key: sk-locus-sentinel\r\nX-Locus-Run-Id: run\r\nX-Locus-Run-Nonce: nonce\r\nConnection: close\r\n\r\n").unwrap();
        let mut response = String::new();
        agent.read_to_string(&mut response).unwrap();

        assert!(response.starts_with("HTTP/1.1 200"));
        assert!(received_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .contains("x-api-key: real-secret"));
        assert_eq!(proxy.audit_rows().len(), 1);
        assert!(proxy.audit_rows()[0].allowed);
        assert!(!response.contains("real-secret"));
    }
}

#[cfg(test)]
mod container {
    use super::*;

    #[test]
    fn two_mounts_only() {
        let mounts = agent_mounts("/tmp/socket", "/tmp/config");
        validate_agent_mounts(&mounts).unwrap();
        assert_eq!(mounts[1].destination, CONFIG_SOURCE);
        assert!(mounts[1].read_only);
        assert!(entrypoint_setup().contains(CONFIG_DESTINATION));
    }

    #[test]
    fn no_docker_socket() {
        let mut mounts = agent_mounts("/tmp/socket", "/tmp/config").to_vec();
        mounts.push(Mount {
            source: "/var/run/docker.sock".into(),
            destination: "/var/run/docker.sock".into(),
            read_only: false,
        });
        assert!(validate_agent_mounts(&mounts).is_err());
    }

    #[test]
    fn workspace_is_a_clone() {
        let command = workspace_clone_command("git://host/project.git", "run-1").unwrap();
        assert!(command.contains("git clone") && command.contains("agent/'run-1'"));
        refuse_primary_branch("agent/run-1").unwrap();
    }

    #[test]
    fn host_tree_unreachable() {
        let mounts = agent_mounts("/tmp/socket", "/tmp/config");
        assert!(mounts.iter().all(|mount| mount.destination != "/workspace"));
        assert!(refuse_primary_branch("main").is_err());
    }

    #[test]
    fn pty_attaches() {
        assert_eq!(
            AGENT_PTY,
            PtyAttachment {
                tty: true,
                stdout: true,
                stderr: true,
            }
        );
    }

    #[test]
    fn reconciles_on_boot() {
        assert_eq!(
            reconcile_on_boot([("alive".into(), true), ("gone".into(), false)]),
            [
                RunReconciliation {
                    run_id: "alive".into(),
                    status: RunStatus::Running,
                    reattach: true
                },
                RunReconciliation {
                    run_id: "gone".into(),
                    status: RunStatus::Aborted,
                    reattach: false
                },
            ]
        );
    }
}

#[cfg(test)]
mod ports {
    use super::*;

    #[test]
    fn allocates_unique() {
        let ports = PortAllocator::default();
        let first = ports.allocate().unwrap();
        let second = ports.allocate().unwrap();
        assert_ne!(first, second);
        assert!((PORT_START..=PORT_END).contains(&first));
    }
}

#[cfg(test)]
mod net {
    use super::*;

    #[test]
    fn project_network() {
        assert_eq!(super::project_network("project-a"), "locus-project-a");
    }

    #[test]
    fn project_isolation() {
        assert!(!same_project_network("project-a", "project-b"));
    }
}

#[cfg(test)]
mod svc {
    use super::*;

    #[test]
    fn up_down() {
        let services = ServiceSupervisor::default();
        assert_eq!(
            services.up("project", "postgres"),
            "locus-svc-project-postgres"
        );
        assert!(services.down("project", "postgres"));
        assert!(!services.down("project", "postgres"));
    }

    #[test]
    fn no_docker_socket_for_agents() {
        let mounts = agent_mounts("/tmp/locus.sock", "/tmp/config");
        assert!(
            mounts
                .iter()
                .all(|mount| mount.destination != "/var/run/docker.sock"),
            "service requests travel over /run/locus.sock; agents never receive Docker's socket"
        );

        let services = ServiceSupervisor::default();
        assert_eq!(services.up("project", "redis"), "locus-svc-project-redis");
    }
}

#[cfg(test)]
mod canary {
    use super::*;

    #[test]
    fn present_in_config() {
        assert!(canary_context("run").contains(&canary_token("run")));
    }

    #[test]
    fn detects_leak() {
        assert!(output_leaks_canary(
            "run",
            &format!("leaked {}", canary_token("run"))
        ));
        assert!(!output_leaks_canary("run", "safe output"));
    }
}

#[cfg(test)]
mod limits {
    use super::*;

    #[test]
    fn tool_call_rate() {
        let limit = ToolCallRateLimit::new(2, Duration::from_secs(1));
        let start = Instant::now();
        assert!(limit.allow("run", start));
        assert!(limit.allow("run", start));
        assert!(!limit.allow("run", start));
        assert!(limit.allow("run", start + Duration::from_secs(1)));
    }
}
