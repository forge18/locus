//! Deterministic failure arbitration for workflow verification.
//!
//! Classification is deliberately data-only. It never owns a model/provider handle: callers
//! supply the bounded evidence the verifier already produced and receive a bounded action.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

use super::planning::PlanTask;

pub const MAX_EVIDENCE_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Bug,
    SpecGap,
    Noise,
    Ambiguity,
}

impl FailureClass {
    pub const ALL: [Self; 4] = [Self::Bug, Self::SpecGap, Self::Noise, Self::Ambiguity];
    pub const fn counts_toward_budget(self) -> bool {
        !matches!(self, Self::Noise)
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::SpecGap => "spec_gap",
            Self::Noise => "noise",
            Self::Ambiguity => "ambiguity",
        }
    }
}

/// Evidence from one failed verification. Flags are supplied by deterministic checks or an
/// already-authored requirement review; no classifier model is involved.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FailureEvidence {
    pub check_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub check_is_flaky: bool,
    pub requirement_missing: bool,
    pub requirement_ambiguous: bool,
}

impl FailureEvidence {
    pub fn new(check_id: impl Into<String>, exit_code: i32) -> Self {
        Self {
            check_id: check_id.into(),
            exit_code,
            ..Self::default()
        }
    }

    fn bounded_text(value: &str) -> String {
        value.chars().take(MAX_EVIDENCE_BYTES).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Iteration {
    pub number: u32,
    pub arbiter_class: Option<FailureClass>,
    pub counts_toward_budget: bool,
}

impl Iteration {
    pub fn new(number: u32) -> Result<Self, ArbiterError> {
        if number == 0 {
            return Err(ArbiterError::InvalidIteration);
        }
        Ok(Self {
            number,
            arbiter_class: None,
            counts_toward_budget: true,
        })
    }

    pub fn record_failure(&mut self, class: FailureClass) {
        self.arbiter_class = Some(class);
        self.counts_toward_budget = class.counts_toward_budget();
    }
}

pub type IterationRecord = Iteration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArbiterAction {
    BugRetry { check_id: String, promoted: bool },
    NoiseRecalibrate { check_id: String },
    SpecGapExit { delta_task: PlanTask },
    AmbiguityRestart { refined_requirement: String },
}

impl ArbiterAction {
    pub fn restarts_implementation(&self) -> bool {
        matches!(self, Self::AmbiguityRestart { .. })
    }
    pub fn exits_workflow(&self) -> bool {
        matches!(self, Self::SpecGapExit { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbiterDecision {
    pub class: FailureClass,
    pub counts_toward_budget: bool,
    pub action: ArbiterAction,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ArbiterError {
    #[error("iteration number must be greater than zero")]
    InvalidIteration,
    #[error("failure check id is required")]
    MissingCheck,
    #[error("failure evidence is too large")]
    EvidenceTooLarge,
    #[error("delta task title is required")]
    MissingDelta,
    #[error("refined requirement is required")]
    MissingRefinement,
}

/// Pure, deterministic classification. Precedence is explicit: noise is a verifier property,
/// then contract ambiguity/gaps, with all remaining failures treated as implementation bugs.
pub fn classify(evidence: &FailureEvidence) -> Result<FailureClass, ArbiterError> {
    if evidence.check_id.trim().is_empty() {
        return Err(ArbiterError::MissingCheck);
    }
    if evidence.stdout.len() > MAX_EVIDENCE_BYTES || evidence.stderr.len() > MAX_EVIDENCE_BYTES {
        return Err(ArbiterError::EvidenceTooLarge);
    }
    Ok(if evidence.check_is_flaky {
        FailureClass::Noise
    } else if evidence.requirement_ambiguous {
        FailureClass::Ambiguity
    } else if evidence.requirement_missing {
        FailureClass::SpecGap
    } else {
        FailureClass::Bug
    })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegressionSet {
    checks: BTreeSet<String>,
}

impl RegressionSet {
    pub fn checks(&self) -> impl Iterator<Item = &str> {
        self.checks.iter().map(String::as_str)
    }
    pub fn contains(&self, check_id: &str) -> bool {
        self.checks.contains(check_id)
    }
    pub fn promote(&mut self, check_id: impl Into<String>) -> Result<bool, ArbiterError> {
        let check_id = check_id.into();
        if check_id.trim().is_empty() {
            return Err(ArbiterError::MissingCheck);
        }
        Ok(self.checks.insert(check_id))
    }
}

/// Apply the bounded action for a failure and record its classification on the iteration.
pub fn arbitrate(
    evidence: &FailureEvidence,
    iteration: &mut Iteration,
    regression: &mut RegressionSet,
    delta_title: Option<&str>,
    refinement: Option<&str>,
) -> Result<ArbiterDecision, ArbiterError> {
    let class = classify(evidence)?;
    iteration.record_failure(class);
    let action = match class {
        FailureClass::Bug => ArbiterAction::BugRetry {
            check_id: evidence.check_id.clone(),
            promoted: regression.promote(evidence.check_id.clone())?,
        },
        FailureClass::Noise => ArbiterAction::NoiseRecalibrate {
            check_id: evidence.check_id.clone(),
        },
        FailureClass::SpecGap => {
            let title = delta_title
                .filter(|title| !title.trim().is_empty())
                .ok_or(ArbiterError::MissingDelta)?;
            ArbiterAction::SpecGapExit {
                delta_task: PlanTask::new(format!("delta:{}", evidence.check_id), title),
            }
        }
        FailureClass::Ambiguity => {
            let requirement = refinement
                .filter(|value| !value.trim().is_empty())
                .ok_or(ArbiterError::MissingRefinement)?;
            ArbiterAction::AmbiguityRestart {
                refined_requirement: FailureEvidence::bounded_text(requirement),
            }
        }
    };
    Ok(ArbiterDecision {
        class,
        counts_toward_budget: class.counts_toward_budget(),
        action,
    })
}

/// Query functions over already-recorded iterations. They perform no I/O and use all classified
/// failures as the denominator, so Noise remains visible in the rate without spending budget.
pub fn spec_gap_rate(iterations: &[Iteration]) -> u8 {
    rate(iterations, FailureClass::SpecGap)
}

pub fn ambiguity_rate(iterations: &[Iteration]) -> u8 {
    rate(iterations, FailureClass::Ambiguity)
}

fn rate(iterations: &[Iteration], class: FailureClass) -> u8 {
    let classified = iterations
        .iter()
        .filter_map(|iteration| iteration.arbiter_class)
        .collect::<Vec<_>>();
    if classified.is_empty() {
        return 0;
    }
    ((classified
        .iter()
        .filter(|candidate| **candidate == class)
        .count()
        * 100)
        / classified.len()) as u8
}

/// Compatibility name for callers that describe this as an arbiter API rather than a service.
pub fn spec_gap_percentage(iterations: &[Iteration]) -> u8 {
    spec_gap_rate(iterations)
}

pub fn ambiguity_detection_rate(iterations: &[Iteration]) -> u8 {
    ambiguity_rate(iterations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_precedence_and_budget() {
        let evidence = FailureEvidence {
            check_id: "test".into(),
            check_is_flaky: true,
            requirement_missing: true,
            ..Default::default()
        };
        assert_eq!(classify(&evidence).unwrap(), FailureClass::Noise);
        assert!(!FailureClass::Noise.counts_toward_budget());
    }
}
