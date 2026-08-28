//! Machine-selected container runtime configuration.
//!
//! Docker remains the default. Docker Sandboxes (`sbx`) is an opt-in backend selected once by
//! the host daemon; a run never silently changes backend while it is alive.

use std::{env, fmt, net::SocketAddr, str::FromStr};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::sandbox::sbx::SbxConfig;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeBackend {
    #[default]
    Docker,
    Sbx,
}

impl RuntimeBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Sbx => "sbx",
        }
    }
}

impl fmt::Display for RuntimeBackend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RuntimeBackend {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "docker" => Ok(Self::Docker),
            "sbx" => Ok(Self::Sbx),
            value => bail!("unsupported container runtime `{value}`; expected docker or sbx"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub backend: RuntimeBackend,
    pub sbx: SbxConfig,
}

impl RuntimeConfig {
    pub fn new(backend: RuntimeBackend) -> Self {
        Self {
            backend,
            sbx: SbxConfig::default(),
        }
    }

    /// Read the machine-level selection. `LOCUS_RUNTIME` is deliberately the only switch;
    /// project and agent input cannot choose a weaker or different backend.
    pub fn from_env() -> Result<Self> {
        let backend = env::var("LOCUS_RUNTIME")
            .ok()
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or_default();
        let sbx = (backend == RuntimeBackend::Sbx)
            .then(SbxConfig::from_env)
            .transpose()?
            .unwrap_or_default();
        Ok(Self { backend, sbx })
    }

    pub fn relay_address(&self) -> Option<SocketAddr> {
        (self.backend == RuntimeBackend::Sbx).then_some(self.sbx.relay_address)
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new(RuntimeBackend::Docker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_is_the_safe_default() {
        assert_eq!(RuntimeConfig::default().backend, RuntimeBackend::Docker);
        assert_eq!(RuntimeBackend::default().as_str(), "docker");
        assert_eq!(
            serde_json::to_string(&RuntimeBackend::Sbx).unwrap(),
            "\"sbx\""
        );
    }

    #[test]
    fn parses_only_the_two_admitted_backends() {
        assert_eq!(
            "docker".parse::<RuntimeBackend>().unwrap(),
            RuntimeBackend::Docker
        );
        assert_eq!(
            "SBX".parse::<RuntimeBackend>().unwrap(),
            RuntimeBackend::Sbx
        );
        assert!("containerd".parse::<RuntimeBackend>().is_err());
    }

    #[test]
    fn sbx_config_keeps_the_relay_on_a_host_bound_endpoint() {
        let config = RuntimeConfig::new(RuntimeBackend::Sbx);
        assert_eq!(
            config.relay_address().unwrap().ip().to_string(),
            "127.0.0.1"
        );
    }
}
