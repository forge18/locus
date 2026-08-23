//! Egress policy tiers and the outbound audit record.
//!
//! A leaf: this module imports nothing from the rest of the crate, so both the proxy that
//! produces an audit and the store that persists one can depend on it without a cycle.

use std::collections::BTreeSet;

use anyhow::Result;
use serde::Serialize;

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

/// Hostname allowlists consumed by the forwarding sidecar. The supervisor builds this from
/// provider endpoints and project package settings; agents never supply a destination policy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DestinationAllowlists {
    model: BTreeSet<String>,
    packages: BTreeSet<String>,
}

impl DestinationAllowlists {
    pub fn new(
        model: impl IntoIterator<Item = impl Into<String>>,
        packages: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            model: model.into_iter().map(Into::into).collect(),
            packages: packages.into_iter().map(Into::into).collect(),
        }
    }

    /// `Open` permits research destinations through the sidecar; it never grants a direct route.
    pub fn permits(&self, tier: EgressTier, target: EgressTarget, host: &str) -> bool {
        match tier {
            EgressTier::None => false,
            EgressTier::Open => !host.is_empty(),
            EgressTier::Model => target == EgressTarget::Model && self.model.contains(host),
            EgressTier::Packages => match target {
                EgressTarget::Model => self.model.contains(host),
                EgressTarget::Package => self.packages.contains(host),
                EgressTarget::Other => false,
            },
        }
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

/// Records one outbound call durably **before** the proxy forwards it.
///
/// The proxy takes a sink rather than a store so the sandbox never names persistence.
/// PLAN.md §Credentials: one audit row per outbound call, from the same code path that
/// decides whether the call is allowed.
pub trait AuditSink: Send + Sync {
    fn record(&self, audit: &OutboundAudit) -> Result<()>;
}

#[cfg(test)]
mod tiers {
    use super::*;

    #[test]
    fn restricts_tiers_to_their_configured_destinations() {
        let allowlists = DestinationAllowlists::new(["api.anthropic.com"], ["registry.npmjs.org"]);

        assert!(allowlists.permits(EgressTier::Model, EgressTarget::Model, "api.anthropic.com"));
        assert!(!allowlists.permits(EgressTier::Model, EgressTarget::Model, "example.test"));
        assert!(allowlists.permits(
            EgressTier::Packages,
            EgressTarget::Package,
            "registry.npmjs.org"
        ));
        assert!(!allowlists.permits(EgressTier::Packages, EgressTarget::Other, "example.test"));
        assert!(allowlists.permits(EgressTier::Open, EgressTarget::Other, "research.example"));
        assert!(!allowlists.permits(EgressTier::None, EgressTarget::Model, "api.anthropic.com"));
    }
}
