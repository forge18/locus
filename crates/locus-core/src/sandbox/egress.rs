//! Egress policy tiers and the outbound audit record.
//!
//! A leaf: this module imports nothing from the rest of the crate, so both the proxy that
//! produces an audit and the store that persists one can depend on it without a cycle.

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
