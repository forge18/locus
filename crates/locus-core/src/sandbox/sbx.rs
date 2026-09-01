//! Docker Sandboxes (`sbx`) runtime backend.
//!
//! `sbx` is deliberately behind the existing [`ContainerRuntime`] boundary. The host owns every
//! command, image import, policy rule, relay, and lifecycle transition; an agent only sees the
//! scratch directory and a TCP endpoint carrying the same authenticated request envelope as the
//! Unix socket path.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::{
    runtime::{
        backend::RuntimeBackend,
        container::{ContainerExecResult, ContainerLaunch, ContainerRuntime, ImageDisposition},
    },
    sandbox::{
        egress::{AuditSink, EgressTarget, EgressTier},
        forward_proxy::{ForwardProxyLaunch, ForwardProxyPolicy},
        image::shell_quote,
        mounts::{validate_agent_mounts, PtyAttachment},
        workspace::refuse_primary_branch,
        CONFIG_SOURCE,
    },
};

pub const DEFAULT_SBX_RELAY_ADDRESS: &str = "127.0.0.1:44001";
pub const DEFAULT_SBX_SCRATCH_ROOT: &str = "/tmp/locus-sbx";
pub const DEFAULT_SBX_GIT_PORT_START: u16 = 44_100;
pub const SBX_GIT_PORT_END: u16 = 44_999;
pub const SBX_CREDENTIAL_PROXY_PORT: u16 = 44_000;
const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const GIT_DAEMON_START_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_COMMAND_ERROR_BYTES: usize = 4_096;
const MAX_COMMAND_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SbxPolicyProfile {
    AllowAll,
    Balanced,
    DenyAll,
}

impl SbxPolicyProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AllowAll => "allow-all",
            Self::Balanced => "balanced",
            Self::DenyAll => "deny-all",
        }
    }
}

impl std::str::FromStr for SbxPolicyProfile {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "allow-all" => Ok(Self::AllowAll),
            "balanced" => Ok(Self::Balanced),
            "deny-all" => Ok(Self::DenyAll),
            value => bail!(
                "unsupported sbx policy profile `{value}`; expected allow-all, balanced, or deny-all"
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SbxConfig {
    pub binary: String,
    pub scratch_root: PathBuf,
    pub relay_address: SocketAddr,
    pub git_port_start: u16,
    pub policy_profile: SbxPolicyProfile,
    pub command_timeout: Duration,
}

impl Default for SbxConfig {
    fn default() -> Self {
        Self {
            binary: "sbx".into(),
            scratch_root: env::temp_dir().join("locus-sbx"),
            relay_address: SocketAddr::from(([127, 0, 0, 1], 44_001)),
            git_port_start: DEFAULT_SBX_GIT_PORT_START,
            policy_profile: SbxPolicyProfile::Balanced,
            command_timeout: DEFAULT_COMMAND_TIMEOUT,
        }
    }
}

impl SbxConfig {
    pub fn from_env() -> Result<Self> {
        let defaults = Self::default();
        let binary = env::var("LOCUS_SBX_BINARY").unwrap_or(defaults.binary);
        let scratch_root = env::var_os("LOCUS_SBX_SCRATCH_ROOT")
            .map(PathBuf::from)
            .unwrap_or(defaults.scratch_root);
        let relay_address = env::var("LOCUS_SBX_RELAY_ADDRESS")
            .unwrap_or_else(|_| DEFAULT_SBX_RELAY_ADDRESS.into())
            .parse()
            .context("LOCUS_SBX_RELAY_ADDRESS must be host:port")?;
        let git_port_start = match env::var("LOCUS_SBX_GIT_PORT_START") {
            Ok(value) => value
                .parse()
                .context("LOCUS_SBX_GIT_PORT_START must be a port")?,
            Err(_) => defaults.git_port_start,
        };
        let policy_profile = match env::var("LOCUS_SBX_POLICY_PROFILE") {
            Ok(value) => value.parse()?,
            Err(_) => defaults.policy_profile,
        };
        let timeout_seconds = match env::var("LOCUS_SBX_COMMAND_TIMEOUT_SECONDS") {
            Ok(value) => value
                .parse::<u64>()
                .context("LOCUS_SBX_COMMAND_TIMEOUT_SECONDS must be an integer")?,
            Err(_) => defaults.command_timeout.as_secs(),
        };
        let config = Self {
            binary,
            scratch_root,
            relay_address,
            git_port_start,
            policy_profile,
            command_timeout: Duration::from_secs(timeout_seconds),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.binary.trim().is_empty() || self.binary.contains('\0') {
            bail!("sbx binary must be a non-empty path")
        }
        if !self.scratch_root.is_absolute() || self.scratch_root == Path::new("/") {
            bail!("sbx scratch root must be an absolute non-root path")
        }
        if !self.relay_address.ip().is_loopback() {
            bail!("sbx relay must bind to a loopback address")
        }
        if self.relay_address.port() == 0 {
            bail!("sbx relay port must be non-zero")
        }
        if self.git_port_start == 0 || self.git_port_start > SBX_GIT_PORT_END {
            bail!("sbx git port start is outside the supported range")
        }
        if self.command_timeout.is_zero() {
            bail!("sbx command timeout must be non-zero")
        }
        Ok(())
    }

    pub fn policy_init_args(&self) -> Vec<String> {
        vec![
            "policy".into(),
            "init".into(),
            self.policy_profile.as_str().into(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SbxCommandOutput {
    pub status_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait SbxCommandRunner: Send {
    fn run(&mut self, program: &str, args: &[String]) -> Result<SbxCommandOutput>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessCommandRunner {
    timeout: Duration,
}

impl ProcessCommandRunner {
    pub fn new(timeout: Duration) -> Result<Self> {
        if timeout.is_zero() {
            bail!("command runner timeout must be non-zero")
        }
        Ok(Self { timeout })
    }
}

impl SbxCommandRunner for ProcessCommandRunner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<SbxCommandOutput> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command
            .spawn()
            .with_context(|| format!("spawn `{program}`"))?;
        let stdout = child.stdout.take().context("capture sbx stdout")?;
        let stderr = child.stderr.take().context("capture sbx stderr")?;
        let stdout_reader = thread::spawn(move || read_command_output(stdout));
        let stderr_reader = thread::spawn(move || read_command_output(stderr));
        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait().context("poll sbx command") {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => {
                    terminate_child(&mut child);
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error);
                }
            }
            if Instant::now() >= deadline {
                terminate_child(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                bail!("`{program}` exceeded its {:?} timeout", self.timeout)
            }
            thread::sleep(Duration::from_millis(10));
        };
        let stdout = stdout_reader
            .join()
            .map_err(|_| anyhow::anyhow!("sbx stdout reader panicked"))?
            .context("collect sbx stdout")?;
        let stderr = stderr_reader
            .join()
            .map_err(|_| anyhow::anyhow!("sbx stderr reader panicked"))?
            .context("collect sbx stderr")?;
        Ok(SbxCommandOutput {
            status_code: status.code().unwrap_or(-1),
            stdout,
            stderr,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SbxSandboxState {
    Missing,
    Stopped,
    Running,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SbxNetworkPolicy {
    pub tier: EgressTier,
    pub relay_port: u16,
    pub git_port: Option<u16>,
    pub service_ports: BTreeSet<u16>,
    pub model_hosts: BTreeSet<String>,
    pub package_hosts: BTreeSet<String>,
}

impl SbxNetworkPolicy {
    pub fn new(
        tier: EgressTier,
        relay_port: u16,
        git_port: Option<u16>,
        model_hosts: impl IntoIterator<Item = String>,
        package_hosts: impl IntoIterator<Item = String>,
        service_ports: impl IntoIterator<Item = u16>,
    ) -> Result<Self> {
        if relay_port == 0 || git_port.is_some_and(|port| port == 0) {
            bail!("sbx policy ports must be non-zero")
        }
        let model_hosts = model_hosts.into_iter().collect::<BTreeSet<_>>();
        let package_hosts = package_hosts.into_iter().collect::<BTreeSet<_>>();
        for host in model_hosts.iter().chain(package_hosts.iter()) {
            validate_policy_host(host)?;
        }
        let service_ports = service_ports.into_iter().collect::<BTreeSet<_>>();
        if service_ports.contains(&0) {
            bail!("sbx service ports must be non-zero")
        }
        Ok(Self {
            tier,
            relay_port,
            git_port,
            service_ports,
            model_hosts,
            package_hosts,
        })
    }

    pub fn resources(&self) -> Vec<String> {
        let mut resources = BTreeSet::new();
        // sbx policy names host-loopback listeners as `localhost`, even though the VM reaches
        // them through `host.docker.internal` (the behavior verified by Spike 4).
        resources.insert(format!("localhost:{}", self.relay_port));
        if let Some(port) = self.git_port {
            resources.insert(format!("localhost:{port}"));
        }
        for port in &self.service_ports {
            resources.insert(format!("localhost:{port}"));
        }
        match self.tier {
            EgressTier::None => {}
            EgressTier::Open => {
                resources.insert("**".into());
            }
            EgressTier::Model => {
                resources.insert(format!("localhost:{SBX_CREDENTIAL_PROXY_PORT}"));
                resources.extend(self.model_hosts.iter().map(|host| network_resource(host)));
            }
            EgressTier::Packages => {
                resources.insert(format!("localhost:{SBX_CREDENTIAL_PROXY_PORT}"));
                resources.extend(self.model_hosts.iter().map(|host| network_resource(host)));
                resources.extend(self.package_hosts.iter().map(|host| network_resource(host)));
            }
        }
        resources.into_iter().collect()
    }
}

pub fn parse_policy_log(
    bytes: &[u8],
    run_id: &str,
    policy: &SbxNetworkPolicy,
) -> Result<Vec<crate::sandbox::egress::OutboundAudit>> {
    if run_id.trim().is_empty() {
        bail!("sbx policy audit requires a run id")
    }
    let value: Value = serde_json::from_slice(bytes).context("decode sbx policy log JSON")?;
    let mut entries = Vec::new();
    collect_policy_log_entries(&value, None, &mut entries);
    let mut audits = Vec::new();
    for (entry, section_allowed) in entries {
        let resource = policy_log_resource(entry)?;
        let allowed = policy_log_allowed(entry, section_allowed)?;
        let count = policy_log_count(entry).unwrap_or(1).clamp(1, 1_024);
        let target = policy_log_target(&resource, policy);
        for _ in 0..count {
            audits.push(crate::sandbox::egress::OutboundAudit {
                run_id: run_id.into(),
                target,
                tier: policy.tier,
                allowed,
                credential_class: "sbx_policy",
            });
        }
    }
    Ok(audits)
}

fn collect_policy_log_entries<'a>(
    value: &'a Value,
    section_allowed: Option<bool>,
    entries: &mut Vec<(&'a serde_json::Map<String, Value>, Option<bool>)>,
) {
    if let Some(object) = value.as_object() {
        if policy_log_resource_value(object).is_some()
            && (section_allowed.is_some()
                || object.contains_key("allowed")
                || object.contains_key("decision")
                || object.contains_key("status"))
        {
            entries.push((object, section_allowed));
        }
        for (key, child) in object {
            let child_section = match key.as_str() {
                "allowed_hosts" => Some(true),
                "blocked_hosts" => Some(false),
                _ => section_allowed,
            };
            collect_policy_log_entries(child, child_section, entries);
        }
    } else if let Some(array) = value.as_array() {
        for child in array {
            collect_policy_log_entries(child, section_allowed, entries);
        }
    }
}

fn policy_log_resource_value(object: &serde_json::Map<String, Value>) -> Option<&str> {
    ["host", "hostname", "resource", "destination"]
        .into_iter()
        .find_map(|key| object.get(key).and_then(Value::as_str))
}

fn policy_log_resource(object: &serde_json::Map<String, Value>) -> Result<String> {
    policy_log_resource_value(object)
        .filter(|resource| !resource.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("sbx policy log entry has no resource"))
}

fn policy_log_allowed(
    object: &serde_json::Map<String, Value>,
    section_allowed: Option<bool>,
) -> Result<bool> {
    if let Some(allowed) = object.get("allowed").and_then(Value::as_bool) {
        return Ok(allowed);
    }
    let Some(decision) = ["decision", "status"]
        .into_iter()
        .find_map(|key| object.get(key).and_then(Value::as_str))
    else {
        return section_allowed
            .ok_or_else(|| anyhow::anyhow!("sbx policy log entry has no decision"));
    };
    match decision.to_ascii_lowercase().as_str() {
        "allow" | "allowed" | "permit" | "permitted" => Ok(true),
        "deny" | "denied" | "block" | "blocked" | "refused" => Ok(false),
        value => bail!("sbx policy log has unknown decision `{value}`"),
    }
}

fn policy_log_count(object: &serde_json::Map<String, Value>) -> Option<u64> {
    ["count", "count_since", "request_count", "requests"]
        .into_iter()
        .find_map(|key| object.get(key).and_then(Value::as_u64))
}

fn policy_log_target(resource: &str, policy: &SbxNetworkPolicy) -> EgressTarget {
    let host = resource
        .rsplit_once(':')
        .filter(|(_, port)| port.parse::<u16>().is_ok())
        .map_or(resource, |(host, _)| host);
    if resource == format!("host.docker.internal:{SBX_CREDENTIAL_PROXY_PORT}")
        || resource == format!("localhost:{SBX_CREDENTIAL_PROXY_PORT}")
    {
        return EgressTarget::Model;
    }
    if policy
        .model_hosts
        .iter()
        .any(|allowed| allowed == host || network_resource(allowed) == resource)
    {
        EgressTarget::Model
    } else if policy
        .package_hosts
        .iter()
        .any(|allowed| allowed == host || network_resource(allowed) == resource)
    {
        EgressTarget::Package
    } else {
        EgressTarget::Other
    }
}

#[derive(Clone)]
struct PreparedLaunch {
    launch: ContainerLaunch,
    scratch: PathBuf,
    remote: Option<String>,
    branch: String,
    git_port: Option<u16>,
    policy: SbxNetworkPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PersistedLaunch {
    image: String,
    command: Vec<String>,
    environment: Vec<String>,
    remote: Option<String>,
    branch: String,
    git_port: Option<u16>,
    policy: SbxNetworkPolicy,
}

pub struct SbxContainerRuntime {
    config: SbxConfig,
    runner: Box<dyn SbxCommandRunner>,
    prepared: BTreeMap<String, PreparedLaunch>,
    git_daemons: BTreeMap<String, Child>,
    audit_sink: Option<Arc<dyn AuditSink>>,
    next_git_port: u16,
}

impl SbxContainerRuntime {
    pub fn connect(config: SbxConfig) -> Result<Self> {
        let timeout = config.command_timeout;
        let mut runtime = Self::with_runner(config, Box::new(ProcessCommandRunner::new(timeout)?))?;
        let output = runtime.run_sbx(vec!["version".into()])?;
        if output.status_code != 0 {
            bail!(
                "sbx runtime is unavailable: {}",
                display_stderr(&output.stderr)
            )
        }
        Ok(runtime)
    }

    pub fn with_runner(config: SbxConfig, runner: Box<dyn SbxCommandRunner>) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            next_git_port: config.git_port_start,
            config,
            runner,
            prepared: BTreeMap::new(),
            git_daemons: BTreeMap::new(),
            audit_sink: None,
        })
    }

    pub fn config(&self) -> &SbxConfig {
        &self.config
    }

    pub fn initialize_policy(&mut self) -> Result<()> {
        self.checked_sbx(
            self.config.policy_init_args(),
            "initialize sbx network policy",
        )
        .map(|_| ())
    }

    pub fn ensure_policy_initialized(&mut self) -> Result<()> {
        let output = self.run_sbx(vec!["policy".into(), "ls".into(), "--json".into()])?;
        if output.status_code != 0 {
            bail!(
                "sbx network policy is not initialized; run `sbx policy init {}`",
                self.config.policy_profile.as_str()
            )
        }
        Ok(())
    }

    pub fn setup(&mut self, images: &[String]) -> Result<Vec<ImageDisposition>> {
        self.ensure_policy_initialized()?;
        self.pre_pull_templates(images.iter().cloned())
    }

    pub fn pre_pull_templates(
        &mut self,
        images: impl IntoIterator<Item = String>,
    ) -> Result<Vec<ImageDisposition>> {
        let images = images.into_iter().collect::<BTreeSet<_>>();
        images
            .into_iter()
            .map(|image| self.build_or_reuse_image(&image))
            .collect()
    }

    pub fn sandbox_state(&mut self, name: &str) -> Result<SbxSandboxState> {
        validate_sbx_name(name)?;
        let output = self.run_sbx(vec!["ls".into(), "--json".into()])?;
        if output.status_code != 0 {
            bail!("read sbx sandbox state: {}", display_stderr(&output.stderr))
        }
        parse_sandbox_state(name, &output.stdout)
    }

    pub fn audit_policy_log(
        &mut self,
        name: &str,
        run_id: &str,
        policy: &SbxNetworkPolicy,
    ) -> Result<Vec<crate::sandbox::egress::OutboundAudit>> {
        let output = self.checked_sbx(
            vec!["policy".into(), "log".into(), name.into(), "--json".into()],
            "read sbx policy log",
        )?;
        let audits = parse_policy_log(&output.stdout, run_id, policy)?;
        if let Some(sink) = self.audit_sink.clone() {
            for audit in &audits {
                sink.record(audit)?;
            }
        }
        Ok(audits)
    }

    pub fn prepare_launch(&mut self, launch: &mut ContainerLaunch) -> Result<()> {
        validate_sbx_name(&launch.name)?;
        validate_agent_mounts(&launch.mounts)?;
        let config_source = launch
            .mounts
            .iter()
            .find(|mount| mount.destination == CONFIG_SOURCE)
            .map(|mount| PathBuf::from(&mount.source))
            .ok_or_else(|| anyhow::anyhow!("sbx launch has no materialized config source"))?;
        let scratch = self.config.scratch_root.join(&launch.name);
        prepare_scratch(&scratch, &config_source)?;

        let mut environment = environment_map(&launch.environment)?;
        let remote = environment.remove("LOCUS_SBX_WORKSPACE_REMOTE");
        let branch = environment
            .remove("LOCUS_SBX_WORKSPACE_BRANCH")
            .unwrap_or_else(|| format!("agent/{}", launch.name));
        if branch.trim().is_empty() {
            bail!("sbx workspace branch must not be empty")
        }
        refuse_primary_branch(&branch)?;
        validate_remote(remote.as_deref())?;
        let git_port = remote
            .as_deref()
            .filter(|remote| is_local_remote(remote))
            .map(|_| self.allocate_git_port())
            .transpose()?;
        let policy =
            policy_from_environment(&environment, self.config.relay_address.port(), git_port)?;

        for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"] {
            environment.remove(key);
        }
        environment.insert("LOCUS_RUNTIME".into(), RuntimeBackend::Sbx.to_string());
        environment.insert(
            "LOCUS_SOCKET_ENDPOINT".into(),
            format!(
                "tcp://host.docker.internal:{}",
                self.config.relay_address.port()
            ),
        );
        environment.insert(
            "LOCUS_CONFIG".into(),
            scratch.join(".locus/config").display().to_string(),
        );
        environment.insert("LOCUS_WORKSPACE".into(), "/workspace".into());
        environment.insert("LOCUS_SBX_SCRATCH".into(), scratch.display().to_string());
        if let Some(git_port) = git_port {
            environment.insert("LOCUS_SBX_GIT_PORT".into(), git_port.to_string());
        }
        launch.environment = environment
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        let prepared = PreparedLaunch {
            launch: launch.clone(),
            scratch,
            remote,
            branch,
            git_port,
            policy,
        };
        persist_launch(&self.config.scratch_root, &prepared)?;
        self.prepared.insert(launch.name.clone(), prepared);
        Ok(())
    }

    pub fn create_args(launch: &ContainerLaunch, scratch: &Path) -> Result<Vec<String>> {
        validate_sbx_name(&launch.name)?;
        if launch.image.trim().is_empty() || launch.image.contains('\0') {
            bail!("sbx launch image must be non-empty")
        }
        if !scratch.is_absolute() {
            bail!("sbx scratch path must be absolute")
        }
        let mut args = vec![
            "create".into(),
            "--quiet".into(),
            "--name".into(),
            launch.name.clone(),
            "--template".into(),
            launch.image.clone(),
        ];
        let mut environment = launch.environment.clone();
        environment.sort();
        for value in environment {
            args.push("--env".into());
            args.push(value);
        }
        if let Some(port) = environment_value(&launch.environment, "LOCUS_PORT")
            .map(|value| value.parse::<u16>())
            .transpose()
            .context("LOCUS_PORT must be a port")?
        {
            if port == 0 {
                bail!("LOCUS_PORT must be non-zero")
            }
            args.push("--publish".into());
            args.push(format!("127.0.0.1:{port}:{port}"));
        }
        args.extend(["shell".into(), scratch.display().to_string()]);
        Ok(args)
    }

    pub fn exec_args(
        name: &str,
        attachment: PtyAttachment,
        script: &str,
        command: &[String],
    ) -> Result<Vec<String>> {
        validate_sbx_name(name)?;
        if script.trim().is_empty() || command.is_empty() {
            bail!("sbx exec requires a setup script and command")
        }
        let mut args = vec!["exec".into()];
        if attachment.tty {
            args.push("-t".into());
        }
        args.push("-i".into());
        args.extend([
            name.into(),
            "/bin/sh".into(),
            "-lc".into(),
            script.into(),
            "locus-agent".into(),
        ]);
        args.extend(command.iter().cloned());
        Ok(args)
    }

    pub fn git_daemon_command(remote: &str, port: u16) -> Result<Vec<String>> {
        if port == 0 || !is_local_remote(remote) {
            bail!("git daemon requires a local bare remote and non-zero port")
        }
        let path = Path::new(remote);
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| anyhow::anyhow!("git remote has no parent directory"))?;
        if !path.is_absolute() || path.file_name().is_none() {
            bail!("git remote must be an absolute repository path")
        }
        let parent = parent.display().to_string();
        Ok(vec![
            "daemon".into(),
            "--reuseaddr".into(),
            "--listen=127.0.0.1".into(),
            format!("--port={port}"),
            format!("--base-path={parent}"),
            "--export-all".into(),
            "--enable=receive-pack".into(),
            parent,
        ])
    }

    pub fn git_remote_url(remote: &str, host: &str, port: u16) -> Result<String> {
        if host.trim().is_empty() || port == 0 {
            bail!("git remote URL requires a host and port")
        }
        if !is_local_remote(remote) {
            let parsed = Url::parse(remote).context("parse git remote URL")?;
            if parsed.scheme() != "git"
                || parsed.host_str().is_none_or(str::is_empty)
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.query().is_some()
                || parsed.fragment().is_some()
            {
                bail!("sbx workspace remote must be a local path or credential-free git URL")
            }
            return Ok(remote.into());
        }
        let repository = Path::new(remote)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty() && !name.contains('\0'))
            .ok_or_else(|| anyhow::anyhow!("git remote repository name is invalid"))?;
        Ok(format!("git://{host}:{port}/{repository}"))
    }

    fn allocate_git_port(&mut self) -> Result<u16> {
        while self.next_git_port <= SBX_GIT_PORT_END {
            let port = self.next_git_port;
            self.next_git_port = self.next_git_port.saturating_add(1);
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return Ok(port);
            }
        }
        bail!("no sbx git-daemon ports remain")
    }

    fn run_sbx(&mut self, args: Vec<String>) -> Result<SbxCommandOutput> {
        let binary = self.config.binary.clone();
        self.runner.run(&binary, &args)
    }

    fn checked_sbx(&mut self, args: Vec<String>, action: &str) -> Result<SbxCommandOutput> {
        let output = self.run_sbx(args)?;
        if output.status_code != 0 {
            bail!("{action} failed: {}", display_stderr(&output.stderr))
        }
        Ok(output)
    }

    fn apply_policy(&mut self, name: &str, policy: &SbxNetworkPolicy) -> Result<()> {
        let resources = policy.resources();
        if resources.is_empty() {
            return Ok(());
        }
        self.checked_sbx(
            vec![
                "policy".into(),
                "allow".into(),
                "network".into(),
                "--sandbox".into(),
                name.into(),
                resources.join(","),
            ],
            "apply sbx sandbox policy",
        )
        .map(|_| ())
    }

    fn launch_git_daemon(&mut self, name: &str, remote: &str, port: u16) -> Result<()> {
        if !is_local_remote(remote) {
            return Ok(());
        }
        let path = Path::new(remote);
        if !path.is_dir() {
            bail!("git remote `{remote}` is not a directory")
        }
        if !path.join("HEAD").is_file() || !path.join("objects").is_dir() {
            bail!("git remote `{remote}` is not a bare repository")
        }
        let args = Self::git_daemon_command(remote, port)?;
        let mut command = Command::new("git");
        command
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command.spawn().context("start host git daemon for sbx")?;
        let deadline = Instant::now() + GIT_DAEMON_START_TIMEOUT;
        loop {
            if let Some(status) = child.try_wait().context("poll host git daemon")? {
                bail!("git daemon exited before port {port} opened ({status})")
            }
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                self.git_daemons.insert(name.into(), child);
                return Ok(());
            }
            if Instant::now() >= deadline {
                terminate_child(&mut child);
                bail!("git daemon did not open port {port} within 2 seconds")
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn stop_git_daemon(&mut self, name: &str) {
        if let Some(mut child) = self.git_daemons.remove(name) {
            terminate_child(&mut child);
        }
    }

    fn audit_container_policy(&mut self, name: &str) -> Result<()> {
        if self.audit_sink.is_none() {
            return Ok(());
        }
        let Some(prepared) = self.prepared.get(name).cloned() else {
            return Ok(());
        };
        let run_id = name.strip_prefix("locus-agent-").unwrap_or(name);
        self.audit_policy_log(name, run_id, &prepared.policy)
            .map(|_| ())
    }

    fn prepared_launch(&mut self, launch: &ContainerLaunch) -> Result<PreparedLaunch> {
        if !self.prepared.contains_key(&launch.name) {
            let mut copy = launch.clone();
            self.prepare_launch(&mut copy)?;
        }
        self.prepared
            .get(&launch.name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("sbx launch was not prepared"))
    }

    fn run_verify(
        &mut self,
        request: &crate::services::workflow::VerifyContainerRequest,
    ) -> Result<crate::services::workflow::VerifyEvidence> {
        let name = request.container_name.clone();
        validate_sbx_name(&name)?;
        self.ensure_policy_initialized()?;
        let scratch = self.config.scratch_root.join(format!("verify-{name}"));
        prepare_empty_scratch(&scratch)?;
        let git_port = is_local_remote(&request.workspace_remote)
            .then(|| self.allocate_git_port())
            .transpose()?;
        if let (Some(port), true) = (git_port, is_local_remote(&request.workspace_remote)) {
            self.launch_git_daemon(&name, &request.workspace_remote, port)?;
        }
        let remote = git_port
            .map(|port| {
                Self::git_remote_url(&request.workspace_remote, "host.docker.internal", port)
            })
            .transpose()?
            .unwrap_or_else(|| request.workspace_remote.clone());
        let launch = ContainerLaunch {
            name: name.clone(),
            image: request.image.clone(),
            command: vec!["/bin/sh".into()],
            entrypoint: String::new(),
            environment: Vec::new(),
            mounts: Vec::new(),
            network: String::new(),
        };
        let result = (|| {
            self.checked_sbx(
                Self::create_args(&launch, &scratch)?,
                "create sbx verification sandbox",
            )?;
            let policy = SbxNetworkPolicy::new(
                EgressTier::None,
                self.config.relay_address.port(),
                git_port,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )?;
            self.apply_policy(&name, &policy)?;
            let clone = workspace_setup_command(&scratch, Some(&remote), &request.branch)?;
            let script = format!("{clone} && {}", request.command);
            let args = Self::exec_args(
                &name,
                PtyAttachment {
                    tty: false,
                    stdout: true,
                    stderr: true,
                },
                &script,
                &["/bin/sh".into()],
            )?;
            let output = self.run_sbx(args)?;
            Ok(crate::services::workflow::VerifyEvidence {
                exit_code: output.status_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                passed: output.status_code == 0,
                command: request.command.clone(),
                container_id: name.clone(),
                verify_node_id: request.verify_node_id.clone(),
            })
        })();
        let removal = self.checked_sbx(
            vec!["rm".into(), "--force".into(), name.clone()],
            "remove sbx verification sandbox",
        );
        self.stop_git_daemon(&name);
        let _ = fs::remove_dir_all(&scratch);
        match (result, removal) {
            (Ok(evidence), Ok(_)) => Ok(evidence),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }
}

impl ContainerRuntime for SbxContainerRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Sbx
    }

    fn attach_audit_sink(&mut self, sink: Arc<dyn AuditSink>) -> Result<()> {
        self.audit_sink = Some(sink);
        Ok(())
    }

    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition> {
        if image.trim().is_empty() || image.contains('\0') {
            bail!("sbx image tag must be non-empty")
        }
        let listed = self.checked_sbx(
            vec!["template".into(), "ls".into(), "--json".into()],
            "list sbx templates",
        )?;
        if template_is_loaded(&listed.stdout, image) {
            return Ok(ImageDisposition::Reused);
        }
        fs::create_dir_all(&self.config.scratch_root)
            .context("create sbx image import scratch root")?;
        let tar_path = self.config.scratch_root.join(format!(
            "image-{}-{}.tar",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let save = self.runner.run(
            "docker",
            &[
                "save".into(),
                "--output".into(),
                tar_path.display().to_string(),
                image.into(),
            ],
        )?;
        if save.status_code != 0 {
            let _ = fs::remove_file(&tar_path);
            bail!(
                "save Docker image for sbx: {}",
                display_stderr(&save.stderr)
            )
        }
        let loaded = self.checked_sbx(
            vec![
                "template".into(),
                "load".into(),
                tar_path.display().to_string(),
            ],
            "load Docker image into sbx",
        );
        let _ = fs::remove_file(&tar_path);
        loaded.map(|_| ImageDisposition::Built)
    }

    fn prepare_container(&mut self, launch: &mut ContainerLaunch) -> Result<()> {
        self.prepare_launch(launch)
    }

    fn start_container(&mut self, launch: &ContainerLaunch) -> Result<()> {
        let prepared = self.prepared_launch(launch)?;
        self.ensure_policy_initialized()?;
        let state = self.sandbox_state(&launch.name)?;
        let new_sandbox = state == SbxSandboxState::Missing;
        let git_started = if let (Some(remote), Some(port)) = (&prepared.remote, prepared.git_port)
        {
            if !self.git_daemons.contains_key(&launch.name) {
                self.launch_git_daemon(&launch.name, remote, port)?;
                true
            } else {
                false
            }
        } else {
            false
        };
        if new_sandbox {
            let create = self.checked_sbx(
                Self::create_args(&prepared.launch, &prepared.scratch)?,
                "create sbx agent sandbox",
            );
            if let Err(error) = create {
                if git_started {
                    self.stop_git_daemon(&launch.name);
                }
                return Err(error);
            }
        }
        if let Err(error) = self.apply_policy(&launch.name, &prepared.policy) {
            let _ = self.checked_sbx(
                vec!["rm".into(), "--force".into(), launch.name.clone()],
                "remove failed sbx agent sandbox",
            );
            self.stop_git_daemon(&launch.name);
            return Err(error);
        }
        if state == SbxSandboxState::Stopped {
            self.checked_sbx(
                vec!["exec".into(), launch.name.clone(), "/bin/true".into()],
                "start stopped sbx agent sandbox",
            )?;
        }
        let remote = prepared
            .git_port
            .map(|port| {
                Self::git_remote_url(
                    prepared.remote.as_deref().unwrap_or_default(),
                    "host.docker.internal",
                    port,
                )
            })
            .transpose()?
            .or(prepared.remote.clone());
        if remote.is_some() {
            let setup =
                workspace_setup_command(&prepared.scratch, remote.as_ref(), &prepared.branch)?;
            self.checked_sbx(
                Self::exec_args(
                    &launch.name,
                    PtyAttachment {
                        tty: false,
                        stdout: false,
                        stderr: false,
                    },
                    &setup,
                    &["/bin/true".into()],
                )?,
                "prepare sbx agent workspace",
            )?;
        }
        Ok(())
    }

    fn stop_container(&mut self, container: &str) -> Result<()> {
        let audit = self.audit_container_policy(container);
        let stopped = self.checked_sbx(
            vec!["stop".into(), container.into()],
            "stop sbx agent sandbox",
        );
        self.stop_git_daemon(container);
        match (audit, stopped) {
            (Err(error), _) => Err(error),
            (_, Err(error)) => Err(error),
            (Ok(()), Ok(_)) => Ok(()),
        }
    }

    fn remove_container(&mut self, container: &str) -> Result<()> {
        let state = self.sandbox_state(container)?;
        let result = if state == SbxSandboxState::Missing {
            Ok(())
        } else {
            self.checked_sbx(
                vec!["rm".into(), "--force".into(), container.into()],
                "remove sbx agent sandbox",
            )
            .map(|_| ())
        };
        self.stop_git_daemon(container);
        if let Some(prepared) = self.prepared.remove(container) {
            let _ = fs::remove_dir_all(prepared.scratch);
        }
        remove_launch_state(&self.config.scratch_root, container);
        result
    }

    fn exec(&mut self, container: &str, command: &[String]) -> Result<ContainerExecResult> {
        if command.is_empty() {
            bail!("container command must not be empty")
        }
        let command = command
            .iter()
            .map(|part| shell_quote(part))
            .collect::<Vec<_>>()
            .join(" ");
        let args = Self::exec_args(
            container,
            PtyAttachment {
                tty: false,
                stdout: true,
                stderr: true,
            },
            &format!("cd /workspace && {command}"),
            &["/bin/true".into()],
        )?;
        let output = self
            .run_sbx(args)
            .context("run command in sbx agent workspace")?;
        Ok(ContainerExecResult {
            status_code: output.status_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    fn container_is_alive(&mut self, container: &str) -> Result<bool> {
        Ok(self.sandbox_state(container)? == SbxSandboxState::Running)
    }

    fn ensure_agent_network(&mut self, network: &str) -> Result<()> {
        if network.trim().is_empty() {
            bail!("sbx agent network name must not be empty")
        }
        Ok(())
    }

    fn ensure_egress_proxy(&mut self, _proxy: &ForwardProxyLaunch) -> Result<()> {
        // sbx's policy daemon is the egress chokepoint; it has no Docker sidecar or shared
        // project network to start.
        Ok(())
    }

    fn release_egress_proxy(&mut self, proxy: &ForwardProxyLaunch, run_id: &str) -> Result<()> {
        ForwardProxyPolicy::remove_from(&proxy.policy_root, run_id)
    }

    fn run_verify_container(
        &mut self,
        request: &crate::services::workflow::VerifyContainerRequest,
    ) -> Result<crate::services::workflow::VerifyEvidence> {
        self.run_verify(request)
    }
}

impl Drop for SbxContainerRuntime {
    fn drop(&mut self) {
        for (_, mut child) in std::mem::take(&mut self.git_daemons) {
            terminate_child(&mut child);
        }
    }
}

impl crate::runtime::boot::BootRuntime for SbxContainerRuntime {
    fn container_is_alive(&mut self, container: &str) -> Result<bool> {
        <Self as ContainerRuntime>::container_is_alive(self, container)
    }

    fn reattach_agent(&mut self, container: &str) -> Result<()> {
        if !<Self as ContainerRuntime>::container_is_alive(self, container)? {
            bail!("agent sandbox `{container}` is not running")
        }
        Ok(())
    }
}

fn policy_from_environment(
    environment: &BTreeMap<String, String>,
    relay_port: u16,
    git_port: Option<u16>,
) -> Result<SbxNetworkPolicy> {
    let tier = environment
        .get("LOCUS_SBX_EGRESS_TIER")
        .map(|value| parse_egress_tier(value))
        .transpose()?
        .unwrap_or(EgressTier::None);
    let model_hosts = split_hosts(environment.get("LOCUS_SBX_MODEL_HOSTS"));
    let package_hosts = split_hosts(environment.get("LOCUS_SBX_PACKAGE_HOSTS"));
    let service_ports = environment
        .get("LOCUS_SBX_SERVICE_PORTS")
        .map(|value| {
            value
                .split(',')
                .filter(|value| !value.trim().is_empty())
                .map(|value| {
                    value
                        .parse()
                        .context("LOCUS_SBX_SERVICE_PORTS must be ports")
                })
                .collect::<Result<Vec<u16>>>()
        })
        .transpose()?
        .unwrap_or_default();
    SbxNetworkPolicy::new(
        tier,
        relay_port,
        git_port,
        model_hosts,
        package_hosts,
        service_ports,
    )
}

fn parse_egress_tier(value: &str) -> Result<EgressTier> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(EgressTier::None),
        "model" => Ok(EgressTier::Model),
        "packages" => Ok(EgressTier::Packages),
        "open" => Ok(EgressTier::Open),
        value => bail!("unsupported sbx egress tier `{value}`"),
    }
}

fn split_hosts(value: Option<&String>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|value| value.split(','))
        .filter(|host| !host.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn network_resource(host: &str) -> String {
    if host
        .rsplit_once(':')
        .is_some_and(|(_, port)| port.parse::<u16>().is_ok())
    {
        host.into()
    } else {
        format!("{host}:443")
    }
}

fn validate_policy_host(host: &str) -> Result<()> {
    if host.trim().is_empty()
        || host.starts_with('-')
        || host.contains([',', '\n', '\r', '\0'])
        || host.chars().any(char::is_whitespace)
    {
        bail!("sbx policy host must be a single non-empty resource")
    }
    Ok(())
}

fn environment_map(environment: &[String]) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for value in environment {
        let (key, value) = value
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("sbx environment entries must be KEY=VALUE"))?;
        if key.trim().is_empty() || key.contains('\0') || value.contains('\0') {
            bail!("sbx environment keys and values must be non-empty and NUL-free")
        }
        values.insert(key.into(), value.into());
    }
    Ok(values)
}

fn environment_value<'a>(environment: &'a [String], key: &str) -> Option<&'a str> {
    environment.iter().find_map(|entry| {
        entry
            .strip_prefix(key)
            .and_then(|value| value.strip_prefix('='))
    })
}

fn prepare_scratch(scratch: &Path, config_source: &Path) -> Result<()> {
    reject_symlink(config_source, "sbx materialized config source")?;
    if !config_source.is_dir() {
        bail!("sbx materialized config source is not a directory")
    }
    reject_symlink(scratch, "sbx scratch path")?;
    fs::create_dir_all(scratch).context("create sbx scratch directory")?;
    let locus = scratch.join(".locus");
    reject_symlink(&locus, "sbx scratch Locus directory")?;
    fs::create_dir_all(&locus).context("create sbx scratch Locus directory")?;
    let config = locus.join("config");
    reject_symlink(&config, "sbx scratch config directory")?;
    fs::create_dir_all(&config).context("create sbx scratch config directory")?;
    clear_directory(&config)?;
    copy_directory_contents(config_source, &config)?;
    let workspace = scratch.join("workspace");
    reject_symlink(&workspace, "sbx workspace directory")?;
    fs::create_dir_all(workspace).context("create sbx workspace directory")?;
    Ok(())
}

fn prepare_empty_scratch(scratch: &Path) -> Result<()> {
    reject_symlink(scratch, "sbx verification scratch")?;
    if scratch.exists() {
        clear_directory(scratch)?;
    } else {
        fs::create_dir_all(scratch).context("create sbx verification scratch")?;
    }
    Ok(())
}

const LAUNCH_STATE_FILE: &str = "launch.json";

fn launch_state_root(scratch_root: &Path) -> PathBuf {
    let suffix = scratch_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("default");
    scratch_root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".locus-sbx-state-{suffix}"))
}

fn launch_state_path(scratch_root: &Path, name: &str) -> Result<PathBuf> {
    validate_sbx_name(name)?;
    Ok(launch_state_root(scratch_root)
        .join(name)
        .join(LAUNCH_STATE_FILE))
}

fn persist_launch(scratch_root: &Path, prepared: &PreparedLaunch) -> Result<()> {
    let path = launch_state_path(scratch_root, &prepared.launch.name)?;
    let root = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sbx launch state has no parent"))?;
    let state_root = root
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sbx launch state has no state root"))?;
    reject_symlink(state_root, "sbx launch state root")?;
    reject_symlink(root, "sbx launch state directory")?;
    fs::create_dir_all(root).context("create sbx launch state directory")?;
    let persisted = PersistedLaunch {
        image: prepared.launch.image.clone(),
        command: prepared.launch.command.clone(),
        environment: prepared.launch.environment.clone(),
        remote: prepared.remote.clone(),
        branch: prepared.branch.clone(),
        git_port: prepared.git_port,
        policy: prepared.policy.clone(),
    };
    let temporary = root.join(format!(
        ".{}.{}.tmp",
        prepared.launch.name,
        uuid::Uuid::new_v4()
    ));
    fs::write(&temporary, serde_json::to_vec(&persisted)?).context("write sbx launch state")?;
    #[cfg(unix)]
    fs::set_permissions(
        &temporary,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )?;
    fs::rename(temporary, path).context("publish sbx launch state")?;
    Ok(())
}

#[cfg(test)]
fn load_persisted_launch(scratch_root: &Path, name: &str) -> Result<PreparedLaunch> {
    let scratch = scratch_root.join(name);
    reject_symlink(&scratch, "sbx scratch path")?;
    if !scratch.is_dir() {
        bail!("sbx scratch directory for `{name}` is missing")
    }
    validate_scratch_layout(&scratch)?;
    let path = launch_state_path(scratch_root, name)?;
    let state_directory = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sbx launch state has no parent"))?;
    let state_root = state_directory
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sbx launch state has no state root"))?;
    reject_symlink(state_root, "sbx launch state root")?;
    reject_symlink(state_directory, "sbx launch state directory")?;
    let persisted: PersistedLaunch = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read sbx launch state `{}`", path.display()))?,
    )
    .context("decode sbx launch state")?;
    validate_remote(persisted.remote.as_deref())?;
    refuse_primary_branch(&persisted.branch)?;
    let launch = ContainerLaunch {
        name: name.into(),
        image: persisted.image,
        command: persisted.command,
        entrypoint: String::new(),
        environment: persisted.environment,
        mounts: Vec::new(),
        network: String::new(),
    };
    Ok(PreparedLaunch {
        launch,
        scratch,
        remote: persisted.remote,
        branch: persisted.branch,
        git_port: persisted.git_port,
        policy: persisted.policy,
    })
}

fn remove_launch_state(scratch_root: &Path, name: &str) {
    if let Ok(path) = launch_state_path(scratch_root, name) {
        let _ = fs::remove_file(&path);
        if let Some(root) = path.parent() {
            let _ = fs::remove_dir(root);
            if let Some(state_root) = root.parent() {
                let _ = fs::remove_dir(state_root);
            }
        }
    }
}

fn reject_symlink(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("{label} must not be a symlink")
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect {label}")),
    }
}

#[cfg(test)]
fn validate_scratch_layout(scratch: &Path) -> Result<()> {
    reject_symlink(&scratch.join(".locus"), "sbx scratch Locus directory")?;
    reject_symlink(
        &scratch.join(".locus/config"),
        "sbx scratch config directory",
    )?;
    reject_symlink(&scratch.join("workspace"), "sbx workspace directory")?;
    Ok(())
}

fn clear_directory(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let path = entry.path();
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            bail!("sbx scratch content must not contain symlinks")
        }
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() {
            bail!("sbx materialized config must not contain symlinks")
        }
        let destination_path = destination.join(entry.file_name());
        if metadata.is_dir() {
            fs::create_dir_all(&destination_path)?;
            copy_directory_contents(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path)?;
        } else {
            bail!(
                "unsupported sbx materialized config entry `{}`",
                source_path.display()
            )
        }
    }
    Ok(())
}

fn workspace_setup_command(
    scratch: &Path,
    remote: Option<&String>,
    branch: &str,
) -> Result<String> {
    refuse_primary_branch(branch)?;
    if branch.trim().is_empty() {
        bail!("sbx workspace branch must not be empty")
    }
    let scratch = shell_quote(&scratch.display().to_string());
    let config = shell_quote(&scratch_path(scratch.as_str(), ".locus/config"));
    let workspace = shell_quote(&scratch_path(scratch.as_str(), "workspace"));
    let mut setup = format!(
        "mkdir -p {scratch}/.locus/config {scratch}/workspace /locus; rm -rf /locus/config /workspace; ln -sfn {config} /locus/config; ln -sfn {workspace} /workspace"
    );
    if let Some(remote) = remote {
        let remote = shell_quote(remote);
        setup.push_str(&format!(
            " && if [ ! -d /workspace/.git ]; then git clone {remote} /workspace; fi && {}",
            workspace_checkout_command(branch)
        ));
    }
    setup.push_str(" && cd /workspace");
    Ok(setup)
}

fn workspace_checkout_command(branch: &str) -> String {
    let branch = shell_quote(branch);
    format!(
        "if git -C /workspace show-ref --verify --quiet refs/heads/{branch}; then git -C /workspace checkout {branch}; elif git -C /workspace show-ref --verify --quiet refs/remotes/origin/{branch}; then git -C /workspace checkout -b {branch} origin/{branch}; else git -C /workspace checkout -b {branch}; fi"
    )
}

fn scratch_path(quoted_scratch: &str, suffix: &str) -> String {
    format!("{quoted_scratch}/{suffix}")
}

fn validate_sbx_name(name: &str) -> Result<()> {
    if name.len() < 2
        || name == "default"
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.'))
        || !name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        bail!("invalid sbx sandbox name `{name}`")
    }
    Ok(())
}

fn validate_remote(remote: Option<&str>) -> Result<()> {
    let Some(remote) = remote else { return Ok(()) };
    if remote.trim().is_empty() || remote.contains('\0') {
        bail!("sbx workspace remote must not be empty")
    }
    if is_local_remote(remote) {
        if !Path::new(remote).is_absolute() {
            bail!("sbx local workspace remote must be absolute")
        }
        return Ok(());
    }
    let parsed = Url::parse(remote).context("parse sbx workspace remote")?;
    if parsed.scheme() != "git"
        || parsed.host_str().is_none_or(str::is_empty)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        bail!("sbx workspace remote must be a local path or credential-free git URL")
    }
    Ok(())
}

fn is_local_remote(remote: &str) -> bool {
    !remote.contains("://")
}

fn read_command_output<R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take((MAX_COMMAND_OUTPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_COMMAND_OUTPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sbx command output exceeded the maximum size",
        ));
    }
    Ok(bytes)
}

fn display_stderr(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();
    if stderr.len() > MAX_COMMAND_ERROR_BYTES {
        let truncated = stderr
            .chars()
            .take(MAX_COMMAND_ERROR_BYTES / 4)
            .collect::<String>();
        format!("{truncated}…")
    } else if stderr.is_empty() {
        "no diagnostic output".into()
    } else {
        stderr
    }
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        let _ = libc::killpg(child.id() as i32, libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub fn parse_sandbox_state(name: &str, bytes: &[u8]) -> Result<SbxSandboxState> {
    validate_sbx_name(name)?;
    let value: Value = serde_json::from_slice(bytes).context("decode sbx ls JSON")?;
    let Some(object) = find_sandbox(&value, name) else {
        return Ok(SbxSandboxState::Missing);
    };
    let state = ["status", "state", "phase"]
        .into_iter()
        .find_map(|key| object.get(key).and_then(Value::as_str))
        .map(|value| value.to_ascii_lowercase());
    if object
        .get("running")
        .and_then(Value::as_bool)
        .is_some_and(|running| running)
        || state
            .as_deref()
            .is_some_and(|state| matches!(state, "running" | "active"))
    {
        return Ok(SbxSandboxState::Running);
    }
    if state.as_deref().is_some_and(|state| {
        matches!(
            state,
            "stopped" | "exited" | "paused" | "created" | "ready" | "inactive"
        )
    }) || object
        .get("running")
        .and_then(Value::as_bool)
        .is_some_and(|running| !running)
    {
        return Ok(SbxSandboxState::Stopped);
    }
    bail!("sbx sandbox `{name}` has an unknown state")
}

fn find_sandbox<'a>(value: &'a Value, name: &str) -> Option<&'a serde_json::Map<String, Value>> {
    if let Some(object) = value.as_object() {
        if object
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|candidate| candidate == name)
        {
            return Some(object);
        }
        for child in object.values() {
            if let Some(found) = find_sandbox(child, name) {
                return Some(found);
            }
        }
    } else if let Some(array) = value.as_array() {
        for child in array {
            if let Some(found) = find_sandbox(child, name) {
                return Some(found);
            }
        }
    }
    None
}

pub fn template_is_loaded(bytes: &[u8], image: &str) -> bool {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    let (repository, tag) = split_image_reference(image);
    let repository = canonical_repository(repository);
    find_template(&value, &repository, tag, image)
}

fn split_image_reference(image: &str) -> (&str, &str) {
    let slash = image.rfind('/').unwrap_or(0);
    image
        .rfind(':')
        .filter(|colon| *colon > slash)
        .map_or((image, "latest"), |colon| {
            (&image[..colon], &image[colon + 1..])
        })
}

fn canonical_repository(repository: &str) -> String {
    if repository.starts_with("docker.io/")
        || repository.contains('.')
        || repository.contains(':')
        || repository == "localhost"
    {
        repository.to_owned()
    } else if repository.contains('/') {
        format!("docker.io/{repository}")
    } else {
        format!("docker.io/library/{repository}")
    }
}

fn find_template(value: &Value, repository: &str, tag: &str, image: &str) -> bool {
    if let Some(object) = value.as_object() {
        if object
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == image)
            || object
                .get("repository")
                .and_then(Value::as_str)
                .zip(object.get("tag").and_then(Value::as_str))
                .is_some_and(|(candidate_repository, candidate_tag)| {
                    canonical_repository(candidate_repository) == repository && candidate_tag == tag
                })
        {
            return true;
        }
        return object
            .values()
            .any(|child| find_template(child, repository, tag, image));
    }
    value.as_array().is_some_and(|array| {
        array
            .iter()
            .any(|child| find_template(child, repository, tag, image))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{runtime::container::ContainerLaunch, sandbox::egress::EgressTarget};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    type RecordedCalls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

    #[derive(Default)]
    struct RecordingRunner {
        calls: RecordedCalls,
        outputs: VecDeque<SbxCommandOutput>,
    }

    impl RecordingRunner {
        fn with_outputs(outputs: impl IntoIterator<Item = SbxCommandOutput>) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                outputs: outputs.into_iter().collect(),
            }
        }
    }

    #[derive(Clone, Default)]
    struct RecordingAuditSink(Arc<Mutex<Vec<crate::sandbox::egress::OutboundAudit>>>);

    impl AuditSink for RecordingAuditSink {
        fn record(&self, audit: &crate::sandbox::egress::OutboundAudit) -> Result<()> {
            self.0.lock().expect("audit sink lock").push(audit.clone());
            Ok(())
        }
    }

    impl SbxCommandRunner for RecordingRunner {
        fn run(&mut self, program: &str, args: &[String]) -> Result<SbxCommandOutput> {
            self.calls
                .lock()
                .expect("recording runner lock")
                .push((program.into(), args.to_vec()));
            Ok(self.outputs.pop_front().unwrap_or(SbxCommandOutput {
                status_code: 0,
                stdout: b"[]".to_vec(),
                stderr: Vec::new(),
            }))
        }
    }

    fn output(stdout: &[u8]) -> SbxCommandOutput {
        SbxCommandOutput {
            status_code: 0,
            stdout: stdout.into(),
            stderr: Vec::new(),
        }
    }

    fn launch() -> ContainerLaunch {
        let root = std::env::temp_dir().join(format!("locus-sbx-config-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create test config");
        fs::write(root.join("AGENTS.md"), "config").expect("write test config");
        ContainerLaunch {
            name: "locus-agent-test".into(),
            image: "locus/agent-test:1".into(),
            command: vec!["pi".into(), "--acp".into()],
            entrypoint: "unused".into(),
            environment: vec![
                "LOCUS_PORT=43000".into(),
                "LOCUS_SBX_WORKSPACE_REMOTE=/var/lib/locus/repos/project.git".into(),
                "LOCUS_SBX_WORKSPACE_BRANCH=agent/test".into(),
                "LOCUS_SBX_EGRESS_TIER=model".into(),
                "LOCUS_SBX_MODEL_HOSTS=api.anthropic.com".into(),
            ],
            mounts: crate::sandbox::mounts::agent_mounts(
                "/run/locus.sock",
                root.display().to_string(),
            )
            .to_vec(),
            network: "locus-project-internal".into(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn process_runner_drains_large_output_without_blocking_the_child() -> Result<()> {
        let mut runner = ProcessCommandRunner::new(Duration::from_secs(2))?;
        let output = runner.run("sh", &["-c".into(), "yes x | head -c 131072".into()])?;
        assert_eq!(output.status_code, 0);
        assert_eq!(output.stdout.len(), 131072);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn process_runner_kills_timed_out_process_groups() -> Result<()> {
        let mut runner = ProcessCommandRunner::new(Duration::from_millis(50))?;
        let error = runner
            .run("sh", &["-c".into(), "sleep 2".into()])
            .err()
            .context("command should time out")?;
        assert!(error.to_string().contains("timeout"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn scratch_and_config_roots_reject_symlinks() -> Result<()> {
        let source =
            std::env::temp_dir().join(format!("locus-sbx-source-{}", uuid::Uuid::new_v4()));
        let scratch =
            std::env::temp_dir().join(format!("locus-sbx-scratch-{}", uuid::Uuid::new_v4()));
        let target =
            std::env::temp_dir().join(format!("locus-sbx-target-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&source)?;
        fs::create_dir_all(&target)?;
        std::os::unix::fs::symlink(&target, &scratch)?;
        assert!(prepare_scratch(&scratch, &source).is_err());
        fs::remove_file(&scratch)?;
        fs::remove_dir_all(&source)?;
        std::os::unix::fs::symlink(&target, &source)?;
        assert!(prepare_scratch(&scratch, &source).is_err());
        fs::remove_file(&source)?;
        fs::create_dir_all(&source)?;
        fs::create_dir_all(&scratch)?;
        std::os::unix::fs::symlink(&target, scratch.join("workspace"))?;
        assert!(prepare_scratch(&scratch, &source).is_err());
        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&scratch);
        let _ = fs::remove_dir_all(&target);
        Ok(())
    }

    #[test]
    fn default_config_is_explicitly_opt_in() {
        assert_eq!(
            SbxConfig::default().policy_profile,
            SbxPolicyProfile::Balanced
        );
        assert_eq!(RuntimeBackend::Sbx.as_str(), "sbx");
    }

    #[test]
    fn an_uninitialized_policy_is_actionable_and_missing_binary_is_not_a_fallback() {
        let runner = RecordingRunner::with_outputs([SbxCommandOutput {
            status_code: 1,
            stdout: Vec::new(),
            stderr: b"policy not initialized".to_vec(),
        }]);
        let mut runtime = SbxContainerRuntime::with_runner(SbxConfig::default(), Box::new(runner))
            .expect("runtime");
        let error = runtime
            .ensure_policy_initialized()
            .expect_err("uninitialized policy is refused");
        assert!(error.to_string().contains("sbx policy init balanced"));

        let missing = SbxContainerRuntime::connect(SbxConfig {
            binary: "/definitely/missing/sbx".into(),
            ..SbxConfig::default()
        });
        assert!(missing.is_err(), "missing sbx must not fall back to Docker");
    }

    #[test]
    fn policy_resources_include_relay_git_services_and_tier_hosts() {
        let policy = SbxNetworkPolicy::new(
            EgressTier::Packages,
            44001,
            Some(44100),
            vec!["api.anthropic.com".into()],
            vec!["registry.npmjs.org".into()],
            [43000, 43001],
        )
        .expect("policy");
        assert_eq!(
            policy.resources(),
            [
                "api.anthropic.com:443",
                "localhost:43000",
                "localhost:43001",
                "localhost:44000",
                "localhost:44001",
                "localhost:44100",
                "registry.npmjs.org:443",
            ]
        );
    }

    #[test]
    fn template_json_matches_repository_and_tag() {
        let templates = br#"{"images":[{"repository":"locus/agent-test","tag":"1"}]}"#;
        assert!(template_is_loaded(templates, "locus/agent-test:1"));
        assert!(!template_is_loaded(templates, "locus/agent-test:2"));
        let sbx_templates =
            br#"{"images":[{"repository":"docker.io/locus/agent-test","tag":"1"}]}"#;
        assert!(template_is_loaded(sbx_templates, "locus/agent-test:1"));
    }

    #[test]
    fn template_import_saves_the_docker_image_then_loads_the_tar() {
        let runner = RecordingRunner::with_outputs([
            output(br#"{"images":[]}"#),
            output(b""),
            output(b"loaded"),
        ]);
        let calls = runner.calls.clone();
        let scratch_root = std::env::temp_dir().join(format!("locus-sbx-{}", uuid::Uuid::new_v4()));
        let mut runtime = SbxContainerRuntime::with_runner(
            SbxConfig {
                scratch_root: scratch_root.clone(),
                ..SbxConfig::default()
            },
            Box::new(runner),
        )
        .expect("runtime");
        assert_eq!(
            runtime
                .build_or_reuse_image("locus/agent-test:1")
                .expect("import image"),
            ImageDisposition::Built
        );
        let calls = calls.lock().expect("calls").clone();
        assert_eq!(calls[0].1, ["template", "ls", "--json"]);
        assert_eq!(calls[1].0, "docker");
        assert_eq!(calls[1].1[0..2], ["save", "--output"]);
        assert_eq!(calls[1].1[3], "locus/agent-test:1");
        assert_eq!(calls[2].1[0..2], ["template", "load"]);
        assert!(!calls
            .iter()
            .any(|(_, args)| args.iter().any(|arg| arg == "--clone")));
        let _ = fs::remove_dir_all(scratch_root);
    }

    #[test]
    fn policy_log_expands_request_counts_and_reads_decisions_not_exit_codes() {
        let policy = SbxNetworkPolicy::new(
            EgressTier::Model,
            44001,
            None,
            vec!["api.anthropic.com".into()],
            Vec::new(),
            Vec::new(),
        )
        .expect("policy");
        let logs = br#"{"logs":[{"host":"api.anthropic.com:443","decision":"allow","count":2},{"resource":"example.test:443","allowed":false}]}"#;
        let audits = parse_policy_log(logs, "run-1", &policy).expect("policy logs");
        assert_eq!(audits.len(), 3);
        assert_eq!(audits[0].target, EgressTarget::Model);
        assert!(audits[0].allowed);
        assert_eq!(audits[2].target, EgressTarget::Other);
        assert!(!audits[2].allowed);
        let actual_shape = br#"{"allowed_hosts":[{"host":"api.anthropic.com:443","count_since":2}],"blocked_hosts":[{"host":"example.test:443","count_since":1}]}"#;
        let actual = parse_policy_log(actual_shape, "run-1", &policy).expect("actual policy logs");
        assert_eq!(actual.len(), 3);
        assert!(actual[0].allowed);
        assert!(!actual[2].allowed);
    }

    #[test]
    fn policy_log_is_forwarded_to_the_shared_audit_sink() {
        let runner = RecordingRunner::with_outputs([output(
            br#"{"blocked_hosts":[{"host":"example.test:80","count_since":1}]}"#,
        )]);
        let mut runtime = SbxContainerRuntime::with_runner(SbxConfig::default(), Box::new(runner))
            .expect("runtime");
        let sink = RecordingAuditSink::default();
        runtime
            .attach_audit_sink(Arc::new(sink.clone()))
            .expect("attach audit sink");
        let policy = SbxNetworkPolicy::new(
            EgressTier::None,
            44001,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("policy");
        let audits = runtime
            .audit_policy_log("locus-agent-test", "run-1", &policy)
            .expect("read policy log");
        assert_eq!(audits.len(), 1);
        assert_eq!(sink.0.lock().expect("audit sink lock").len(), 1);
        assert!(!audits[0].allowed);
    }

    #[test]
    fn state_json_distinguishes_running_stopped_and_missing() {
        assert_eq!(
            parse_sandbox_state(
                "agent",
                br#"{"sandboxes":[{"name":"agent","status":"running"}]}"#
            )
            .expect("running state"),
            SbxSandboxState::Running
        );
        assert_eq!(
            parse_sandbox_state(
                "agent",
                br#"{"sandboxes":[{"name":"agent","state":"stopped"}]}"#
            )
            .expect("stopped state"),
            SbxSandboxState::Stopped
        );
        assert_eq!(
            parse_sandbox_state("agent", br#"{"sandboxes":[]}"#).expect("missing state"),
            SbxSandboxState::Missing
        );
    }

    #[test]
    fn create_never_uses_clone_and_publishes_the_run_port() {
        let launch = launch();
        let args = SbxContainerRuntime::create_args(&launch, Path::new("/tmp/locus-sbx/test"))
            .expect("create args");
        assert!(!args.iter().any(|argument| argument == "--clone"));
        assert!(!args.iter().any(|argument| argument.contains("docker.sock")));
        assert!(!args.iter().any(|argument| argument == "/run/locus.sock"));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--publish", "127.0.0.1:43000:43000"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--template", "locus/agent-test:1"]));
        let config_source = launch
            .mounts
            .iter()
            .find(|mount| mount.destination == CONFIG_SOURCE)
            .expect("config mount");
        let _ = fs::remove_dir_all(&config_source.source);
    }

    #[test]
    fn workspace_setup_creates_or_reuses_the_run_branch() -> Result<()> {
        let remote = "git://host.docker.internal:44100/project.git".to_owned();
        let command = workspace_setup_command(
            Path::new("/tmp/locus-sbx/run"),
            Some(&remote),
            "agent/run-1",
        )?;
        assert!(command.contains("git clone 'git://host.docker.internal:44100/project.git'"));
        assert!(command.contains("show-ref --verify --quiet refs/heads/'agent/run-1'"));
        assert!(command.contains("checkout -b 'agent/run-1'"));
        assert!(command.ends_with("cd /workspace"));
        Ok(())
    }

    #[test]
    fn git_daemon_enables_receive_pack_without_shelling_through_a_string() {
        let args =
            SbxContainerRuntime::git_daemon_command("/var/lib/locus/repos/project.git", 44100)
                .expect("git daemon args");
        assert!(args.contains(&"--enable=receive-pack".into()));
        assert!(!args.iter().any(|argument| argument.contains("--clone")));
    }

    #[test]
    fn acp_exec_keeps_interactive_stdin_without_allocating_a_tty() {
        let args = SbxContainerRuntime::exec_args(
            "locus-agent-test",
            PtyAttachment {
                tty: false,
                stdout: true,
                stderr: true,
            },
            "exec \"$@\"",
            &["pi".into(), "--acp".into()],
        )
        .expect("ACP exec args");
        assert!(args.contains(&"-i".into()));
        assert!(!args.contains(&"-t".into()));
    }

    #[test]
    fn prepare_replaces_mounts_with_one_scratch_and_tcp_endpoint() {
        let mut runtime = SbxContainerRuntime::with_runner(
            SbxConfig {
                scratch_root: std::env::temp_dir()
                    .join(format!("locus-sbx-{}", uuid::Uuid::new_v4())),
                ..SbxConfig::default()
            },
            Box::new(RecordingRunner::default()),
        )
        .expect("runtime");
        let mut launch = launch();
        runtime.prepare_launch(&mut launch).expect("prepare launch");
        assert!(
            environment_value(&launch.environment, "LOCUS_SOCKET_ENDPOINT")
                .is_some_and(|value| value == "tcp://host.docker.internal:44001")
        );
        assert!(launch
            .environment
            .iter()
            .all(|value| !value.starts_with("HTTP_PROXY=") && !value.starts_with("HTTPS_PROXY=")));
        let prepared = runtime.prepared.get(&launch.name).expect("prepared launch");
        assert_eq!(prepared.policy.git_port, Some(44100));
        remove_launch_state(&runtime.config.scratch_root, &launch.name);
        let _ = fs::remove_dir_all(&runtime.config.scratch_root);
        let config_source = launch
            .mounts
            .iter()
            .find(|mount| mount.destination == CONFIG_SOURCE)
            .expect("config mount");
        let _ = fs::remove_dir_all(&config_source.source);
    }

    #[test]
    fn persisted_launch_metadata_supports_boot_reattachment() {
        let scratch_root = std::env::temp_dir().join(format!("locus-sbx-{}", uuid::Uuid::new_v4()));
        let mut first = SbxContainerRuntime::with_runner(
            SbxConfig {
                scratch_root: scratch_root.clone(),
                ..SbxConfig::default()
            },
            Box::new(RecordingRunner::default()),
        )
        .expect("first runtime");
        let mut launch = launch();
        launch
            .environment
            .retain(|value| !value.starts_with("LOCUS_SBX_WORKSPACE_REMOTE="));
        first.prepare_launch(&mut launch).expect("persist launch");
        let restored = load_persisted_launch(&scratch_root, &launch.name).expect("load launch");
        assert_eq!(restored.launch.image, launch.image);
        assert_eq!(restored.branch, "agent/test");
        remove_launch_state(&scratch_root, &launch.name);
        let _ = fs::remove_dir_all(scratch_root);
        let config_source = launch
            .mounts
            .iter()
            .find(|mount| mount.destination == CONFIG_SOURCE)
            .expect("config mount");
        let _ = fs::remove_dir_all(&config_source.source);
    }

    #[test]
    fn lifecycle_uses_create_policy_stop_and_remove_commands() {
        let runner = RecordingRunner::with_outputs([
            output(br#"{"sandboxes":[]}"#),
            output(br#"{"rules":[]}"#),
            output(br#"{"sandboxes":[]}"#),
            output(br#"{"sandboxes":[]}"#),
            output(br#"{"sandboxes":[]}"#),
            output(br#"{"sandboxes":[{"name":"locus-agent-test","status":"running"}]}"#),
        ]);
        let calls = runner.calls.clone();
        let mut runtime = SbxContainerRuntime::with_runner(
            SbxConfig {
                scratch_root: std::env::temp_dir()
                    .join(format!("locus-sbx-{}", uuid::Uuid::new_v4())),
                ..SbxConfig::default()
            },
            Box::new(runner),
        )
        .expect("runtime");
        let mut launch = launch();
        // The test remote is intentionally not a directory, so omit it from the fake lifecycle.
        launch
            .environment
            .retain(|value| !value.starts_with("LOCUS_SBX_WORKSPACE_REMOTE="));
        runtime.prepare_launch(&mut launch).expect("prepare launch");
        runtime.start_container(&launch).expect("start sandbox");
        runtime.stop_container(&launch.name).expect("stop sandbox");
        runtime
            .remove_container(&launch.name)
            .expect("remove sandbox");
        let calls = calls.lock().expect("calls").clone();
        assert!(calls
            .iter()
            .any(|(_, args)| args.first() == Some(&"create".into())));
        assert!(calls
            .iter()
            .any(|(_, args)| args.first() == Some(&"policy".into())));
        assert!(calls
            .iter()
            .any(|(_, args)| args.first() == Some(&"stop".into())));
        assert!(calls
            .iter()
            .any(|(_, args)| args.first() == Some(&"rm".into())));
        remove_launch_state(&runtime.config.scratch_root, &launch.name);
        let _ = fs::remove_dir_all(&runtime.config.scratch_root);
        let config_source = launch
            .mounts
            .iter()
            .find(|mount| mount.destination == CONFIG_SOURCE)
            .expect("config mount");
        let _ = fs::remove_dir_all(&config_source.source);
    }

    #[test]
    fn policy_profile_and_remote_are_validated() {
        assert!("balanced".parse::<SbxPolicyProfile>().is_ok());
        assert!("allow-all".parse::<SbxPolicyProfile>().is_ok());
        assert!(SbxContainerRuntime::git_remote_url(
            "https://user:secret@example.test/repo.git",
            "host.docker.internal",
            44100,
        )
        .is_err());
        assert_eq!(
            SbxContainerRuntime::git_remote_url(
                "/var/lib/locus/repos/project.git",
                "host.docker.internal",
                44100,
            )
            .expect("git URL"),
            "git://host.docker.internal:44100/project.git"
        );
    }

    #[test]
    fn policy_parse_uses_no_network_for_none() {
        let policy = policy_from_environment(
            &BTreeMap::from([("LOCUS_SBX_EGRESS_TIER".into(), "none".into())]),
            44001,
            None,
        )
        .expect("policy");
        assert_eq!(policy.resources(), ["localhost:44001"]);
        assert!(!policy.tier.allows(EgressTarget::Other));
    }
}
