//! Registered host adapters that make harnesses launchable.

use std::collections::BTreeSet;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AdapterVersion {
    pub identity: String,
    pub version: String,
}

impl AdapterVersion {
    pub fn new(identity: impl Into<String>, version: impl Into<String>) -> Result<Self> {
        let adapter = Self {
            identity: identity.into(),
            version: version.into(),
        };
        if adapter.identity.trim().is_empty() || adapter.version.trim().is_empty() {
            bail!("adapter identity and version must not be empty")
        }
        Ok(adapter)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdapterRegistry(BTreeSet<AdapterVersion>);

impl<const N: usize> From<[AdapterVersion; N]> for AdapterRegistry {
    fn from(adapters: [AdapterVersion; N]) -> Self {
        Self(adapters.into_iter().collect())
    }
}

impl AdapterRegistry {
    pub fn contains(&self, identity: &str, version: &str) -> bool {
        self.0.contains(&AdapterVersion {
            identity: identity.into(),
            version: version.into(),
        })
    }
}
