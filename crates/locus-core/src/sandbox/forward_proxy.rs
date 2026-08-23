//! The project forwarding-proxy sidecar contract.
//!
//! Agent networks are Docker `internal` networks.  The only container that also joins an
//! egress network is this Locus-built sidecar, so setting a proxy variable is not the security
//! boundary: a client that ignores it still has no route off the internal network.

use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::egress::{DestinationAllowlists, EgressTier};
use super::ports::{project_egress_network, project_internal_network};

pub const FORWARD_PROXY_IMAGE: &str = "locus/egress-proxy:latest";
pub const FORWARD_PROXY_ALIAS: &str = "locus-egress-proxy";
pub const FORWARD_PROXY_PORT: u16 = 3128;

/// Host-owned per-run policy consumed by the vendored forwarding sidecar.
///
/// It is a file rather than agent configuration: it lives in a host-only directory mounted
/// read-only into the sidecar, and is re-read before every authorization decision.  This makes
/// capability revocation effective without restarting a project sidecar used by other runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForwardProxyPolicy {
    pub run_id: String,
    pub nonce: String,
    pub tier: EgressTier,
    pub model_hosts: BTreeSet<String>,
    pub package_hosts: BTreeSet<String>,
}

impl ForwardProxyPolicy {
    pub fn new(
        run_id: impl Into<String>,
        nonce: impl Into<String>,
        tier: EgressTier,
        allowlists: &DestinationAllowlists,
    ) -> Result<Self> {
        let run_id = run_id.into();
        let nonce = nonce.into();
        if run_id.trim().is_empty() || nonce.trim().is_empty() {
            bail!("forward proxy policy requires a run id and nonce")
        }
        Ok(Self {
            run_id,
            nonce,
            tier,
            model_hosts: allowlists.model_hosts().cloned().collect(),
            package_hosts: allowlists.package_hosts().cloned().collect(),
        })
    }

    /// `None` does not receive a policy file and therefore cannot authenticate to the sidecar.
    pub fn enabled(&self) -> bool {
        self.tier != EgressTier::None
    }

    /// URL used by standard HTTP clients.  The nonce is a run-scoped capability, never a
    /// credential; the sidecar additionally requires the run id and does not trust a hostname.
    pub fn proxy_url(&self) -> String {
        format!(
            "http://{}:{}@{}:{FORWARD_PROXY_PORT}",
            percent_encode(&self.run_id),
            percent_encode(&self.nonce),
            FORWARD_PROXY_ALIAS,
        )
    }

    pub fn agent_environment(&self) -> Vec<(String, String)> {
        if !self.enabled() {
            return Vec::new();
        }
        let proxy = self.proxy_url();
        vec![
            ("HTTP_PROXY".into(), proxy.clone()),
            ("HTTPS_PROXY".into(), proxy.clone()),
            ("ALL_PROXY".into(), proxy),
            // Do not exempt Docker's host gateway or any other destination from the proxy.
            ("NO_PROXY".into(), String::new()),
        ]
    }

    /// Atomically deliver the policy to the sidecar's host-only policy volume.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        fs::create_dir_all(root).context("create forwarding proxy policy directory")?;
        let temporary = root.join(format!(".{}.tmp", self.run_id));
        let path = policy_path(root, &self.run_id)?;
        let mut file = fs::File::create(&temporary).context("create forwarding proxy policy")?;
        writeln!(file, "{}", self.nonce).context("write forwarding proxy nonce")?;
        writeln!(file, "{}", tier_name(self.tier)).context("write forwarding proxy tier")?;
        writeln!(
            file,
            "{}",
            self.model_hosts
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        )
        .context("write forwarding proxy model allowlist")?;
        writeln!(
            file,
            "{}",
            self.package_hosts
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        )
        .context("write forwarding proxy package allowlist")?;
        file.sync_all().context("sync forwarding proxy policy")?;
        fs::rename(temporary, path).context("publish forwarding proxy policy")
    }

    pub fn remove_from(root: &Path, run_id: &str) -> Result<()> {
        let path = policy_path(root, run_id)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).context("remove forwarding proxy policy"),
        }
    }
}

/// Docker resources for the sidecar owned by one project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForwardProxyLaunch {
    pub project_id: String,
    pub name: String,
    pub image: String,
    pub internal_network: String,
    pub egress_network: String,
    pub policy_root: PathBuf,
}

impl ForwardProxyLaunch {
    pub fn for_project(project_id: &str, policy_root: PathBuf) -> Result<Self> {
        validate_project_id(project_id)?;
        Ok(Self {
            project_id: project_id.into(),
            name: format!("locus-egress-proxy-{project_id}"),
            image: FORWARD_PROXY_IMAGE.into(),
            internal_network: project_internal_network(project_id),
            egress_network: project_egress_network(project_id),
            policy_root,
        })
    }
}

pub fn policy_path(root: &Path, run_id: &str) -> Result<PathBuf> {
    if run_id.is_empty()
        || run_id.contains('/')
        || run_id.contains('\\')
        || run_id == "."
        || run_id == ".."
    {
        bail!("invalid forwarding proxy run id")
    }
    Ok(root.join(run_id))
}

pub fn policy_directory_is_empty(root: &Path) -> Result<bool> {
    match fs::read_dir(root) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error).context("read forwarding proxy policy directory"),
    }
}

fn validate_project_id(project_id: &str) -> Result<()> {
    if project_id.is_empty()
        || !project_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("project id is not safe for Docker resource names")
    }
    Ok(())
}

fn tier_name(tier: EgressTier) -> &'static str {
    match tier {
        EgressTier::None => "none",
        EgressTier::Model => "model",
        EgressTier::Packages => "packages",
        EgressTier::Open => "open",
    }
}

fn percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[cfg(test)]
mod forwarded {
    use super::*;
    use std::fs;

    #[test]
    fn forwards_only_authenticated_egress_capable_runs_through_the_sidecar() {
        let policy = ForwardProxyPolicy::new(
            "run-1",
            "nonce",
            EgressTier::Packages,
            &DestinationAllowlists::new(["api.anthropic.com"], ["registry.npmjs.org"]),
        )
        .unwrap();
        assert_eq!(
            policy.proxy_url(),
            "http://run-1:nonce@locus-egress-proxy:3128"
        );
        assert!(policy
            .agent_environment()
            .iter()
            .any(|(key, _)| key == "HTTPS_PROXY"));
        assert!(ForwardProxyPolicy::new(
            "run-2",
            "nonce",
            EgressTier::None,
            &DestinationAllowlists::default()
        )
        .unwrap()
        .agent_environment()
        .is_empty());
    }

    #[test]
    fn policy_delivery_is_host_only_and_revocable() {
        let root =
            std::env::temp_dir().join(format!("locus-proxy-policy-{}", uuid::Uuid::new_v4()));
        let policy = ForwardProxyPolicy::new(
            "run-1",
            "nonce",
            EgressTier::Model,
            &DestinationAllowlists::new(["api.anthropic.com"], std::iter::empty::<&str>()),
        )
        .unwrap();
        policy.write_to(&root).unwrap();
        let delivered = fs::read_to_string(root.join("run-1")).unwrap();
        assert!(delivered.contains("api.anthropic.com"));
        assert!(!policy_directory_is_empty(&root).unwrap());
        ForwardProxyPolicy::remove_from(&root, "run-1").unwrap();
        assert!(policy_directory_is_empty(&root).unwrap());
        let _ = fs::remove_dir_all(root);
    }
}
