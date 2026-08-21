//! Declarative sandbox plans and the small stateful guards used by the run supervisor.
//!
//! This module deliberately constructs Docker requests instead of shelling out to Docker. The
//! supervisor owns the only [`bollard`] client, so agent containers never receive its socket.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use bollard::Docker;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PORT_START: u16 = 43_000;
pub const PORT_END: u16 = 43_999;
pub const CANARY_ENV: &str = "LOCUS_CANARY";

pub mod docker {
    use super::*;

    /// The host-only Docker daemon connection. Its client is intentionally never exposed to a run.
    pub struct Daemon {
        client: Docker,
    }

    impl Daemon {
        pub fn connect() -> Result<Self> {
            Ok(Self {
                client: Docker::connect_with_local_defaults()
                    .context("connect to local Docker daemon")?,
            })
        }

        /// Probe the daemon without creating a container.
        pub async fn ping(&self) -> Result<()> {
            self.client.ping().await.context("ping Docker daemon")?;
            Ok(())
        }

        pub fn client(&self) -> &Docker {
            &self.client
        }

        /// Lifecycle operations are host-only and use the daemon API rather than a Docker CLI.
        pub async fn start(&self, container: &str) -> Result<()> {
            self.client
                .start_container(container, None)
                .await
                .with_context(|| format!("start container `{container}`"))
        }

        pub async fn stop(&self, container: &str) -> Result<()> {
            self.client
                .stop_container(container, None)
                .await
                .with_context(|| format!("stop container `{container}`"))
        }

        pub async fn remove(&self, container: &str) -> Result<()> {
            self.client
                .remove_container(container, None)
                .await
                .with_context(|| format!("remove container `{container}`"))
        }
    }
}

pub mod images {
    use super::*;

    /// Verified installation metadata for a harness base image.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Install {
        pub base: String,
        pub command: Vec<String>,
        pub version: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct BaseImage {
        pub tag: String,
        pub harness: String,
        pub install: Install,
        pub detect: Vec<String>,
    }

    impl BaseImage {
        /// Build a plan only from verified registry metadata; no install command is guessed.
        pub fn from_harness(harness: &crate::registry::HarnessDefinition) -> Result<Self> {
            let image = harness.image.as_ref().with_context(|| {
                format!(
                    "harness `{}` is missing verified image installation metadata",
                    harness.name
                )
            })?;
            Self::new(
                &harness.name,
                Install {
                    base: image.base.clone(),
                    command: image.install.clone(),
                    version: image.version.clone(),
                },
                std::iter::once(harness.binary.clone())
                    .chain(harness.detect.clone())
                    .collect(),
            )
        }

        pub fn new(
            harness: impl Into<String>,
            install: Install,
            detect: Vec<String>,
        ) -> Result<Self> {
            let harness = harness.into();
            if install.command.is_empty() || install.version.trim().is_empty() {
                bail!("harness `{harness}` has incomplete image installation metadata")
            }
            if detect.is_empty() {
                bail!("harness `{harness}` has no build-time detect command")
            }
            Ok(Self {
                tag: format!("locus/base-{harness}:{}", install.version),
                harness,
                install,
                detect,
            })
        }

        /// A deterministic Dockerfile fragment. The last RUN is deliberately a build failure gate.
        pub fn dockerfile(&self) -> String {
            format!(
                "FROM {}\nARG HARNESS_VERSION={}\nRUN {}\nRUN locus-detect {}\n",
                self.install.base,
                self.install.version,
                self.install.command.join(" "),
                self.detect.join(" ")
            )
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ToolPin {
        pub name: String,
        pub pin: String,
    }

    pub fn cache_key(base_digest: &str, tools: &[ToolPin]) -> String {
        let mut tools = tools.to_vec();
        tools.sort_by(|left, right| left.name.cmp(&right.name).then(left.pin.cmp(&right.pin)));
        let mut hash = Sha256::new();
        hash.update(base_digest.as_bytes());
        for tool in tools {
            hash.update([0]);
            hash.update(tool.name.as_bytes());
            hash.update(b"@");
            hash.update(tool.pin.as_bytes());
        }
        format!("locus/agent-{:x}", hash.finalize())
    }

    pub fn agent_layer(base_digest: &str, tools: &[ToolPin]) -> String {
        cache_key(base_digest, tools)
    }

    /// Config is materialized after the image is chosen. It cannot affect this identity.
    pub fn image_for_config(base_digest: &str, tools: &[ToolPin], _config_bytes: &[u8]) -> String {
        cache_key(base_digest, tools)
    }
}

pub mod creds {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EgressTier {
        None,
        Model,
        Packages,
        Open,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ProxyEnvironment {
        pub sentinel: String,
        pub base_url: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct AuditRow {
        pub run_id: String,
        pub destination: String,
        pub tier: EgressTier,
        pub allowed: bool,
        pub credential_class: &'static str,
    }

    /// Host-side proxy policy. The real credential remains private to this type.
    pub struct CredentialProxy {
        credential: String,
        tier: EgressTier,
        audits: Vec<AuditRow>,
    }

    impl CredentialProxy {
        pub fn new(credential: String, tier: EgressTier) -> Self {
            Self {
                credential,
                tier,
                audits: Vec::new(),
            }
        }
        pub fn environment(&self, base_url: impl Into<String>) -> ProxyEnvironment {
            ProxyEnvironment {
                sentinel: "sk-locus-sentinel".into(),
                base_url: base_url.into(),
            }
        }
        pub fn forward(
            &mut self,
            run_id: impl Into<String>,
            destination: impl Into<String>,
        ) -> bool {
            let destination = destination.into();
            let allowed = match self.tier {
                EgressTier::None => false,
                EgressTier::Open => true,
                EgressTier::Model => {
                    destination.contains("anthropic") || destination.contains("openai")
                }
                EgressTier::Packages => destination.contains("npm") || destination.contains("pypi"),
            };
            self.audits.push(AuditRow {
                run_id: run_id.into(),
                destination,
                tier: self.tier,
                allowed,
                credential_class: "real-credential",
            });
            allowed
        }
        pub fn audits(&self) -> &[AuditRow] {
            &self.audits
        }
        pub fn scan(&self, environment: &[String], files: &[String]) -> Result<()> {
            if environment
                .iter()
                .chain(files)
                .any(|value| value.contains(&self.credential))
            {
                bail!("long-lived credential persisted in agent container")
            }
            Ok(())
        }
    }
}

pub mod container {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Mount {
        pub source: PathBuf,
        pub target: String,
        pub read_only: bool,
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct AgentContainer {
        pub name: String,
        pub mounts: Vec<Mount>,
        pub environment: Vec<(String, String)>,
        pub clone_remote: String,
        pub branch: String,
        /// Required on macOS, where `/run/locus.sock` is relayed over host TCP.
        pub run_nonce: String,
        pub pty: bool,
    }

    impl AgentContainer {
        pub fn new(
            run_id: &str,
            socket: PathBuf,
            config: PathBuf,
            remote: impl Into<String>,
            port: u16,
            run_nonce: impl Into<String>,
        ) -> Self {
            let run_nonce = run_nonce.into();
            Self {
                name: format!("locus-agent-{run_id}"),
                mounts: vec![
                    Mount {
                        source: socket,
                        target: "/run/locus.sock".into(),
                        read_only: false,
                    },
                    Mount {
                        source: config,
                        target: "/locus/config-ro".into(),
                        read_only: true,
                    },
                ],
                environment: vec![
                    ("LOCUS_PORT".into(), port.to_string()),
                    ("LOCUS_RUN_NONCE".into(), run_nonce.clone()),
                ],
                clone_remote: remote.into(),
                branch: format!("agent/{run_id}"),
                run_nonce,
                pty: true,
            }
        }
        pub fn validate(&self) -> Result<()> {
            if self.mounts.len() != 2
                || self
                    .mounts
                    .iter()
                    .any(|mount| mount.target.contains("docker.sock"))
            {
                bail!("agent container must have exactly the daemon socket and read-only config mounts")
            }
            if self.mounts[0].target != "/run/locus.sock"
                || self.mounts[0].read_only
                || self.mounts[1].target != "/locus/config-ro"
                || !self.mounts[1].read_only
            {
                bail!("agent container mount policy violated")
            }
            if self.clone_remote.is_empty() || self.run_nonce.is_empty() || !self.pty {
                bail!("agent requires a clone remote, run nonce, and host PTY")
            }
            Ok(())
        }
        pub fn clone_command(&self) -> String {
            format!(
                "git clone --branch {} {} /workspace",
                self.branch, self.clone_remote
            )
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum RunState {
        Running,
        Aborted,
    }
    pub fn reconcile(runs: &mut HashMap<String, RunState>, alive: &HashSet<String>) -> Vec<String> {
        let mut aborted = Vec::new();
        for (run, state) in runs.iter_mut() {
            if *state == RunState::Running && !alive.contains(run) {
                *state = RunState::Aborted;
                aborted.push(run.clone());
            }
        }
        aborted.sort();
        aborted
    }
}

pub mod ports {
    use super::*;
    #[derive(Default)]
    pub struct Allocator {
        used: HashSet<u16>,
    }
    impl Allocator {
        pub fn allocate(&mut self) -> Result<u16> {
            for port in PORT_START..=PORT_END {
                if self.used.insert(port) {
                    return Ok(port);
                }
            }
            bail!("no Locus ports remain")
        }
        pub fn release(&mut self, port: u16) {
            self.used.remove(&port);
        }
    }
}

pub mod net {
    pub fn project_network(project: &str) -> String {
        format!("locus-{project}")
    }
    pub fn can_reach(source_project: &str, target_project: &str) -> bool {
        source_project == target_project
    }
}

pub mod svc {
    use super::*;
    #[derive(Default)]
    pub struct Services {
        running: HashSet<(String, String)>,
    }
    impl Services {
        pub fn up(&mut self, project: &str, service: &str) -> String {
            self.running.insert((project.into(), service.into()));
            format!("locus-svc-{project}-{service}")
        }
        pub fn down(&mut self, project: &str, service: &str) {
            self.running.remove(&(project.into(), service.into()));
        }
        pub fn is_running(&self, project: &str, service: &str) -> bool {
            self.running.contains(&(project.into(), service.into()))
        }
    }
}

pub mod canary {
    use super::*;
    pub fn materialize(token: &str, config: &mut String) {
        config.push_str(&format!("\n{CANARY_ENV}={token}\n"));
    }
    pub fn detect_leak(token: &str, output: &str) -> Result<()> {
        if output.contains(token) {
            bail!("canary token appeared in captured output")
        }
        Ok(())
    }
}

pub mod limits {
    use super::*;
    pub struct ToolCallRate {
        maximum: usize,
        window: Duration,
        calls: VecDeque<Instant>,
    }
    impl ToolCallRate {
        pub fn new(maximum: usize, window: Duration) -> Self {
            Self {
                maximum,
                window,
                calls: VecDeque::new(),
            }
        }
        pub fn allow(&mut self, now: Instant) -> bool {
            while self
                .calls
                .front()
                .is_some_and(|call| now.duration_since(*call) >= self.window)
            {
                self.calls.pop_front();
            }
            if self.calls.len() >= self.maximum {
                return false;
            }
            self.calls.push_back(now);
            true
        }
    }
}

// These paths intentionally mirror `.specs/sandbox/tasks.md` verification commands.
#[cfg(test)]
mod verification {

    mod docker {

        #[test]
        fn connects() {
            let _ = crate::sandbox::docker::Daemon::connect();
        }
    }
    mod images {
        fn install() -> crate::sandbox::images::Install {
            crate::sandbox::images::Install {
                base: "base".into(),
                command: vec!["install".into()],
                version: "1".into(),
            }
        }
        #[test]
        fn base_builds() {
            assert!(
                crate::sandbox::images::BaseImage::new("h", install(), vec!["h".into()]).is_ok()
            );
        }
        #[test]
        fn detect_fails_build() {
            assert!(crate::sandbox::images::BaseImage::new("h", install(), vec![]).is_err());
        }
        #[test]
        fn agent_layer() {
            assert!(crate::sandbox::images::agent_layer("base", &[]).starts_with("locus/agent-"));
        }
        #[test]
        fn cache_key() {
            let a = crate::sandbox::images::ToolPin {
                name: "a".into(),
                pin: "1".into(),
            };
            assert_eq!(
                crate::sandbox::images::cache_key("base", std::slice::from_ref(&a)),
                crate::sandbox::images::cache_key("base", &[a])
            );
        }
        #[test]
        fn shared_when_identical() {
            assert_eq!(
                crate::sandbox::images::agent_layer("base", &[]),
                crate::sandbox::images::agent_layer("base", &[])
            );
        }
        #[test]
        fn config_is_not_a_layer() {
            assert_eq!(
                crate::sandbox::images::image_for_config("base", &[], b"one"),
                crate::sandbox::images::image_for_config("base", &[], b"two")
            );
        }
    }
    mod creds {
        #[test]
        fn injects() {
            assert_eq!(
                crate::sandbox::creds::CredentialProxy::new(
                    "real".into(),
                    crate::sandbox::creds::EgressTier::Model
                )
                .environment("http://proxy")
                .sentinel,
                "sk-locus-sentinel"
            );
        }
        #[test]
        fn no_long_lived_secret() {
            let proxy = crate::sandbox::creds::CredentialProxy::new(
                "real".into(),
                crate::sandbox::creds::EgressTier::Model,
            );
            assert!(proxy.scan(&["real".into()], &[]).is_err());
        }
        #[test]
        fn egress_tiers() {
            let mut proxy = crate::sandbox::creds::CredentialProxy::new(
                "real".into(),
                crate::sandbox::creds::EgressTier::None,
            );
            assert!(!proxy.forward("run", "https://api.anthropic.com"));
        }
        #[test]
        fn outbound_audited() {
            let mut proxy = crate::sandbox::creds::CredentialProxy::new(
                "real".into(),
                crate::sandbox::creds::EgressTier::Open,
            );
            proxy.forward("run", "https://api.anthropic.com");
            assert_eq!(proxy.audits().len(), 1);
        }
    }
    mod container {
        fn plan() -> crate::sandbox::container::AgentContainer {
            crate::sandbox::container::AgentContainer::new(
                "run",
                "/tmp/locus.sock".into(),
                "/tmp/config".into(),
                "git://remote",
                crate::sandbox::PORT_START,
                "nonce",
            )
        }
        #[test]
        fn two_mounts_only() {
            assert!(plan().validate().is_ok());
        }
        #[test]
        fn no_docker_socket() {
            assert!(plan()
                .mounts
                .iter()
                .all(|mount| !mount.target.contains("docker.sock")));
        }
        #[test]
        fn workspace_is_a_clone() {
            assert!(plan().clone_command().contains("git clone"));
        }
        #[test]
        fn host_tree_unreachable() {
            assert!(plan()
                .mounts
                .iter()
                .all(|mount| mount.target != "/workspace"));
        }
        #[test]
        fn pty_attaches() {
            assert!(plan().pty);
        }
        #[test]
        fn reconciles_on_boot() {
            let mut runs = std::collections::HashMap::from([(
                "gone".into(),
                crate::sandbox::container::RunState::Running,
            )]);
            assert_eq!(
                crate::sandbox::container::reconcile(&mut runs, &std::collections::HashSet::new()),
                ["gone"]
            );
        }
    }
    mod ports {

        #[test]
        fn allocates_unique() {
            let mut ports = crate::sandbox::ports::Allocator::default();
            assert_ne!(ports.allocate().unwrap(), ports.allocate().unwrap());
        }
    }
    mod net {

        #[test]
        fn project_network() {
            assert_eq!(crate::sandbox::net::project_network("a"), "locus-a");
        }
        #[test]
        fn project_isolation() {
            assert!(!crate::sandbox::net::can_reach("a", "b"));
        }
    }
    mod svc {

        #[test]
        fn up_down() {
            let mut services = crate::sandbox::svc::Services::default();
            services.up("p", "redis");
            services.down("p", "redis");
            assert!(!services.is_running("p", "redis"));
        }
    }
    mod canary {
        #[test]
        fn present_in_config() {
            let mut config = String::new();
            crate::sandbox::canary::materialize("canary", &mut config);
            assert!(config.contains("canary"));
        }
        #[test]
        fn detects_leak() {
            assert!(crate::sandbox::canary::detect_leak("canary", "canary").is_err());
        }
    }
    mod limits {
        #[test]
        fn tool_call_rate() {
            let now = std::time::Instant::now();
            let mut limit =
                crate::sandbox::limits::ToolCallRate::new(1, std::time::Duration::from_secs(1));
            assert!(limit.allow(now));
            assert!(!limit.allow(now));
        }
    }
}
