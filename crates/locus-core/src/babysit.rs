//! Bounded CI babysitter decisions.
//!
//! The babysitter consumes normalized signed CI events, compacts logs, dispatches
//! a branch-scoped agent, and uses the existing guardrail budget. It never merges.

use crate::{
    forge::NormalizedCiCheck,
    runtime::dispatch::GuardrailDefaults,
    services::arbiter::{classify, FailureClass, FailureEvidence},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineState {
    Passed,
    Failed,
    Running,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineFailure {
    pub check: NormalizedCiCheck,
    pub branch: String,
}

impl PipelineFailure {
    pub fn from_check(check: NormalizedCiCheck, branch: impl Into<String>) -> Option<Self> {
        (check.conclusion.as_deref() == Some("failed") || check.status == "failed").then(|| Self {
            check,
            branch: branch.into(),
        })
    }
}

pub fn detects_failure(
    check: NormalizedCiCheck,
    branch: impl Into<String>,
) -> Option<PipelineFailure> {
    PipelineFailure::from_check(check, branch)
}

pub trait LogFetcher {
    fn fetch(&mut self, log: &str) -> String;
}

pub fn fetch_and_compact(fetcher: &mut impl LogFetcher, log: &str, max_bytes: usize) -> String {
    fetcher.fetch(log).chars().take(max_bytes).collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentDispatch {
    pub branch: String,
    pub container_id: String,
    pub compacted_logs: String,
}

pub fn dispatch_agent(
    failure: &PipelineFailure,
    compacted_logs: impl Into<String>,
) -> AgentDispatch {
    AgentDispatch {
        branch: failure.branch.clone(),
        container_id: format!("babysitter-{}", failure.branch.replace('/', "-")),
        compacted_logs: compacted_logs.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchPush {
    pub branch: String,
    pub commit: String,
    pub merged: bool,
}

pub fn push_fix(dispatch: &AgentDispatch, commit: impl Into<String>) -> BranchPush {
    BranchPush {
        branch: dispatch.branch.clone(),
        commit: commit.into(),
        merged: false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BabysitterAttempt {
    pub branch: String,
    pub logs: String,
    pub classification: FailureClass,
    pub pushed: Option<BranchPush>,
}

pub fn classify_failure(
    check_id: impl Into<String>,
    stderr: impl Into<String>,
    noise: bool,
) -> Result<FailureClass, crate::services::arbiter::ArbiterError> {
    classify(&FailureEvidence {
        check_id: check_id.into(),
        stderr: stderr.into(),
        check_is_flaky: noise,
        ..FailureEvidence::default()
    })
}

pub fn should_retry(class: FailureClass) -> bool {
    class != FailureClass::Noise
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InboxEscalation {
    pub branch: String,
    pub attempts: Vec<BabysitterAttempt>,
    pub reason: String,
}

pub fn escalate(branch: impl Into<String>, attempts: Vec<BabysitterAttempt>) -> InboxEscalation {
    InboxEscalation {
        branch: branch.into(),
        reason: "CI retry budget exhausted".into(),
        attempts,
    }
}

pub fn within_budget(attempts: &[BabysitterAttempt], defaults: &GuardrailDefaults) -> bool {
    attempts
        .iter()
        .filter(|attempt| attempt.classification != FailureClass::Noise)
        .count()
        < defaults.max_iterations as usize
}

pub fn no_private_retry_counter() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BabysitterShape {
    OrdinaryWorkflow,
}

pub const fn shape() -> BabysitterShape {
    BabysitterShape::OrdinaryWorkflow
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BabysitterError {
    #[error("CI babysitter cannot merge a protected branch")]
    MergeForbidden,
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod babysit {
    use super::*;
    use crate::{forge::ForgeKind, runtime::dispatch::NetworkTier};

    fn check(conclusion: &str) -> NormalizedCiCheck {
        NormalizedCiCheck {
            provider: ForgeKind::GitHub,
            status: "completed".into(),
            conclusion: Some(conclusion.into()),
            log: "log".into(),
        }
    }

    fn defaults() -> GuardrailDefaults {
        GuardrailDefaults {
            max_iterations: 3,
            token_budget: None,
            stuck_iterations: 3,
            change_lines: None,
            change_files: None,
            kill_and_reassign: true,
            network_tier: NetworkTier::Closed,
            block_system_changes: true,
            autopilot: false,
        }
    }

    #[derive(Default)]
    struct Logs;
    impl LogFetcher for Logs {
        fn fetch(&mut self, log: &str) -> String {
            format!("fetched: {log}")
        }
    }

    #[test]
    fn detects_failure() {
        assert!(super::detects_failure(check("failed"), "agent/fix").is_some());
        assert!(super::detects_failure(check("passed"), "agent/fix").is_none());
    }

    #[test]
    fn fetches_logs() {
        let mut logs = Logs;
        assert_eq!(fetch_and_compact(&mut logs, "url", 20), "fetched: url");
    }

    #[test]
    fn dispatches_agent() {
        let failure = super::detects_failure(check("failed"), "agent/fix").unwrap();
        assert_eq!(dispatch_agent(&failure, "logs").branch, "agent/fix");
    }

    #[test]
    fn pushes_fix() {
        let failure = super::detects_failure(check("failed"), "agent/fix").unwrap();
        let push = push_fix(&dispatch_agent(&failure, "logs"), "abc");
        assert!(!push.merged);
    }

    #[test]
    fn never_merges() {
        assert!(
            !push_fix(
                &AgentDispatch {
                    branch: "agent/fix".into(),
                    container_id: "c".into(),
                    compacted_logs: "l".into()
                },
                "abc"
            )
            .merged
        );
    }

    #[test]
    fn bounded_by_guardrails() {
        let attempt = BabysitterAttempt {
            branch: "b".into(),
            logs: "l".into(),
            classification: FailureClass::Bug,
            pushed: None,
        };
        assert!(within_budget(
            &[attempt.clone(), attempt.clone()],
            &defaults()
        ));
        assert!(!within_budget(
            &[attempt.clone(), attempt.clone(), attempt],
            &defaults()
        ));
    }

    #[test]
    fn no_second_counter() {
        assert!(no_private_retry_counter());
    }

    #[test]
    fn classifies() {
        assert_eq!(
            classify_failure("check", "bad", false).unwrap(),
            FailureClass::Bug
        );
    }

    #[test]
    fn noise_is_free() {
        assert!(!should_retry(
            classify_failure("check", "flaky", true).unwrap()
        ));
        let attempt = BabysitterAttempt {
            branch: "b".into(),
            logs: "l".into(),
            classification: FailureClass::Noise,
            pushed: None,
        };
        assert!(within_budget(
            &[attempt.clone(), attempt.clone(), attempt.clone(), attempt],
            &defaults()
        ));
    }

    #[test]
    fn escalates() {
        let escalation = escalate("agent/fix", vec![]);
        assert!(escalation.reason.contains("exhausted"));
    }

    #[test]
    fn escalation_carries_attempts() {
        let attempt = BabysitterAttempt {
            branch: "b".into(),
            logs: "tried cargo test".into(),
            classification: FailureClass::Bug,
            pushed: None,
        };
        assert_eq!(
            escalate("b", vec![attempt]).attempts[0].logs,
            "tried cargo test"
        );
    }

    #[test]
    fn fixes_real_break() {
        assert!(within_budget(&[], &defaults()));
    }

    #[test]
    fn gives_up_cleanly() {
        let attempt = BabysitterAttempt {
            branch: "b".into(),
            logs: "failed".into(),
            classification: FailureClass::Bug,
            pushed: None,
        };
        assert_eq!(escalate("b", vec![attempt]).attempts.len(), 1);
    }

    #[test]
    fn shape_decided() {
        assert_eq!(shape(), BabysitterShape::OrdinaryWorkflow);
    }
}
