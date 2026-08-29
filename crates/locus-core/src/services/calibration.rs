//! Human-gated calibration proposals derived from arbiter classifications.
//!
//! The retro pass reads after a watermark, clusters by recurring check and class,
//! and only places bounded proposals in the reflection queue. Nothing mutates a
//! regression set, wiki concept, or interview rule until an explicit acceptance.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ids::ProjectId,
    services::{
        arbiter::{FailureClass, RegressionSet},
        planning::specialization_concept,
        telemetry::Usage,
        wiki::WikiEvent,
    },
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    pub stream_pos: u64,
    pub task_id: String,
    pub check_id: String,
    pub class: FailureClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Watermark(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetroAgent {
    pub max_classifications: usize,
}

impl Default for RetroAgent {
    fn default() -> Self {
        Self {
            max_classifications: 64,
        }
    }
}

impl RetroAgent {
    pub fn read_since<'a>(
        &self,
        classifications: &'a [Classification],
        watermark: Watermark,
    ) -> Vec<&'a Classification> {
        classifications
            .iter()
            .filter(|classification| classification.stream_pos > watermark.0)
            .take(self.max_classifications)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringCluster {
    pub check_id: String,
    pub class: FailureClass,
    pub task_ids: Vec<String>,
}

pub fn recurring_clusters(classifications: &[&Classification]) -> Vec<RecurringCluster> {
    let mut grouped = BTreeMap::<(String, FailureClass), BTreeSet<String>>::new();
    for classification in classifications {
        grouped
            .entry((classification.check_id.clone(), classification.class))
            .or_default()
            .insert(classification.task_id.clone());
    }
    grouped
        .into_iter()
        .filter(|(_, task_ids)| task_ids.len() > 1)
        .map(|((check_id, class), task_ids)| RecurringCluster {
            check_id,
            class,
            task_ids: task_ids.into_iter().collect(),
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalibrationProposal {
    BugRegression {
        check_id: String,
        tasks: Vec<String>,
    },
    SpecGapClause {
        check_id: String,
        clause: String,
        tasks: Vec<String>,
    },
    NoiseRecalibration {
        check_id: String,
        tasks: Vec<String>,
    },
    AmbiguityInterviewRule {
        check_id: String,
        rule: String,
        tasks: Vec<String>,
    },
}

impl CalibrationProposal {
    pub fn signature(&self) -> String {
        match self {
            Self::BugRegression { check_id, .. } => format!("bug:{check_id}"),
            Self::SpecGapClause { check_id, .. } => format!("spec_gap:{check_id}"),
            Self::NoiseRecalibration { check_id, .. } => format!("noise:{check_id}"),
            Self::AmbiguityInterviewRule { check_id, .. } => format!("ambiguity:{check_id}"),
        }
    }

    pub fn class(&self) -> FailureClass {
        match self {
            Self::BugRegression { .. } => FailureClass::Bug,
            Self::SpecGapClause { .. } => FailureClass::SpecGap,
            Self::NoiseRecalibration { .. } => FailureClass::Noise,
            Self::AmbiguityInterviewRule { .. } => FailureClass::Ambiguity,
        }
    }
}

pub fn proposals_for_clusters(clusters: &[RecurringCluster]) -> Vec<CalibrationProposal> {
    clusters
        .iter()
        .map(|cluster| match cluster.class {
            FailureClass::Bug => CalibrationProposal::BugRegression {
                check_id: cluster.check_id.clone(),
                tasks: cluster.task_ids.clone(),
            },
            FailureClass::SpecGap => CalibrationProposal::SpecGapClause {
                check_id: cluster.check_id.clone(),
                clause: format!("Clarify the requirement tested by {}.", cluster.check_id),
                tasks: cluster.task_ids.clone(),
            },
            FailureClass::Noise => CalibrationProposal::NoiseRecalibration {
                check_id: cluster.check_id.clone(),
                tasks: cluster.task_ids.clone(),
            },
            FailureClass::Ambiguity => CalibrationProposal::AmbiguityInterviewRule {
                check_id: cluster.check_id.clone(),
                rule: format!(
                    "Ask an explicit question about {} before implementation.",
                    cluster.check_id
                ),
                tasks: cluster.task_ids.clone(),
            },
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub enum AcceptedCalibration {
    Regression { check_id: String },
    SpecGapConcept { event: WikiEvent },
    NoiseRecalibrated { check_id: String },
    AmbiguityRule { rule: String },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CalibrationError {
    #[error("reflection proposal index is invalid")]
    InvalidProposal,
    #[error("calibration proposal was already rejected")]
    RejectedProposal,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReflectionQueue {
    pending: Vec<CalibrationProposal>,
    rejected: BTreeSet<String>,
}

impl ReflectionQueue {
    pub fn enqueue(&mut self, proposals: impl IntoIterator<Item = CalibrationProposal>) {
        for proposal in proposals {
            let signature = proposal.signature();
            if !self.rejected.contains(&signature)
                && !self
                    .pending
                    .iter()
                    .any(|current| current.signature() == signature)
            {
                self.pending.push(proposal);
            }
        }
    }

    pub fn pending(&self) -> &[CalibrationProposal] {
        &self.pending
    }

    pub fn reject(&mut self, index: usize) -> Result<(), CalibrationError> {
        let proposal = self
            .pending
            .get(index)
            .ok_or(CalibrationError::InvalidProposal)?;
        self.rejected.insert(proposal.signature());
        self.pending.remove(index);
        Ok(())
    }

    pub fn accept(
        &mut self,
        index: usize,
        project_id: ProjectId,
    ) -> Result<AcceptedCalibration, CalibrationError> {
        let proposal = self
            .pending
            .get(index)
            .cloned()
            .ok_or(CalibrationError::InvalidProposal)?;
        self.pending.remove(index);
        Ok(match proposal {
            CalibrationProposal::BugRegression { check_id, .. } => {
                AcceptedCalibration::Regression { check_id }
            }
            CalibrationProposal::SpecGapClause { clause, .. } => {
                AcceptedCalibration::SpecGapConcept {
                    event: specialization_concept(project_id, clause),
                }
            }
            CalibrationProposal::NoiseRecalibration { check_id, .. } => {
                AcceptedCalibration::NoiseRecalibrated { check_id }
            }
            CalibrationProposal::AmbiguityInterviewRule { rule, .. } => {
                AcceptedCalibration::AmbiguityRule { rule }
            }
        })
    }

    pub fn rejected(&self, proposal: &CalibrationProposal) -> bool {
        self.rejected.contains(&proposal.signature())
    }
}

pub fn apply_bug_acceptance(
    regression: &mut RegressionSet,
    accepted: &AcceptedCalibration,
) -> Result<bool, crate::services::arbiter::ArbiterError> {
    match accepted {
        AcceptedCalibration::Regression { check_id } => regression.promote(check_id.clone()),
        _ => Ok(false),
    }
}

pub fn specialization_injected(confidence: f32, threshold: f32) -> bool {
    confidence >= threshold
}

pub fn default_specialization_threshold() -> f32 {
    0.8
}

/// Return the cache-read share of reported input tokens. Missing or zero input
/// is unknown rather than a zero-rate claim.
pub fn usage_cache_rate(usage: &Usage) -> Option<f64> {
    let input = usage.input?;
    let cache_read = usage.cache_read?;
    (input > 0).then(|| cache_read as f64 / input as f64)
}

/// The paired-run acceptance primitive. A context-policy arm may not reduce the
/// cache-read share of its input; incomplete telemetry fails closed.
pub fn cache_rate_criterion(baseline: &Usage, candidate: &Usage) -> bool {
    match (usage_cache_rate(baseline), usage_cache_rate(candidate)) {
        (Some(baseline), Some(candidate)) => candidate >= baseline,
        _ => false,
    }
}

pub fn cache_rate_non_regression(baseline: &Usage, candidate: &Usage) -> bool {
    cache_rate_criterion(baseline, candidate)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheRateArm {
    pub label: String,
    pub usage: Vec<Usage>,
}

impl CacheRateArm {
    pub fn new(label: impl Into<String>, usage: impl IntoIterator<Item = Usage>) -> Self {
        Self {
            label: label.into(),
            usage: usage.into_iter().collect(),
        }
    }

    pub fn cache_rate(&self) -> Option<f64> {
        let mut input = 0_u64;
        let mut cache_read = 0_u64;
        for usage in &self.usage {
            let current_input = usage.input?;
            let current_cache_read = usage.cache_read?;
            if current_input == 0 {
                return None;
            }
            input = input.checked_add(current_input)?;
            cache_read = cache_read.checked_add(current_cache_read)?;
        }
        (input > 0).then(|| cache_read as f64 / input as f64)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedContextRuns {
    pub baseline: CacheRateArm,
    pub candidate: CacheRateArm,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CacheRateDecision {
    Pass { baseline: f64, candidate: f64 },
    Violation { baseline: f64, candidate: f64 },
    InsufficientData,
}

impl CacheRateDecision {
    pub fn is_non_regression(self) -> bool {
        matches!(self, Self::Pass { .. })
    }
}

pub fn paired_cache_rate_criterion(comparison: &PairedContextRuns) -> CacheRateDecision {
    match (
        comparison.baseline.cache_rate(),
        comparison.candidate.cache_rate(),
    ) {
        (Some(baseline), Some(candidate)) if candidate >= baseline => CacheRateDecision::Pass {
            baseline,
            candidate,
        },
        (Some(baseline), Some(candidate)) => CacheRateDecision::Violation {
            baseline,
            candidate,
        },
        _ => CacheRateDecision::InsufficientData,
    }
}

#[cfg(test)]
mod calibrate {
    use super::*;

    fn classifications() -> Vec<Classification> {
        vec![
            Classification {
                stream_pos: 1,
                task_id: "a".into(),
                check_id: "cargo test".into(),
                class: FailureClass::Bug,
            },
            Classification {
                stream_pos: 2,
                task_id: "b".into(),
                check_id: "cargo test".into(),
                class: FailureClass::Bug,
            },
            Classification {
                stream_pos: 3,
                task_id: "c".into(),
                check_id: "ports".into(),
                class: FailureClass::SpecGap,
            },
            Classification {
                stream_pos: 4,
                task_id: "d".into(),
                check_id: "ports".into(),
                class: FailureClass::SpecGap,
            },
        ]
    }

    #[test]
    fn retro_agent() {
        assert_eq!(RetroAgent::default().max_classifications, 64);
    }

    #[test]
    fn watermark() {
        let records = classifications();
        let read = RetroAgent::default().read_since(&records, Watermark(2));
        assert_eq!(read.len(), 2);
        assert!(read.iter().all(|record| record.stream_pos > 2));
    }

    #[test]
    fn clusters() {
        let records = classifications();
        let references = records.iter().collect::<Vec<_>>();
        assert_eq!(recurring_clusters(&references).len(), 2);
    }

    #[test]
    fn one_proposal_per_cluster() {
        let records = classifications();
        let references = records.iter().collect::<Vec<_>>();
        assert_eq!(
            proposals_for_clusters(&recurring_clusters(&references)).len(),
            2
        );
    }

    #[test]
    fn bug_proposal() {
        let proposal = proposals_for_clusters(&[RecurringCluster {
            check_id: "check".into(),
            class: FailureClass::Bug,
            task_ids: vec!["a".into(), "b".into()],
        }]);
        assert!(
            matches!(&proposal[0], CalibrationProposal::BugRegression { check_id, .. } if check_id == "check")
        );
    }

    #[test]
    fn spec_gap_proposal() {
        let proposal = proposals_for_clusters(&[RecurringCluster {
            check_id: "check".into(),
            class: FailureClass::SpecGap,
            task_ids: vec!["a".into(), "b".into()],
        }]);
        assert!(matches!(
            &proposal[0],
            CalibrationProposal::SpecGapClause { .. }
        ));
    }

    #[test]
    fn noise_proposal() {
        let proposal = proposals_for_clusters(&[RecurringCluster {
            check_id: "check".into(),
            class: FailureClass::Noise,
            task_ids: vec!["a".into(), "b".into()],
        }]);
        assert!(matches!(
            &proposal[0],
            CalibrationProposal::NoiseRecalibration { .. }
        ));
    }

    #[test]
    fn ambiguity_proposal() {
        let proposal = proposals_for_clusters(&[RecurringCluster {
            check_id: "check".into(),
            class: FailureClass::Ambiguity,
            task_ids: vec!["a".into(), "b".into()],
        }]);
        assert!(matches!(
            &proposal[0],
            CalibrationProposal::AmbiguityInterviewRule { .. }
        ));
    }

    #[test]
    fn exactly_four_types() {
        let classes = [
            FailureClass::Bug,
            FailureClass::SpecGap,
            FailureClass::Noise,
            FailureClass::Ambiguity,
        ];
        assert_eq!(
            classes
                .iter()
                .map(|class| class.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn review_queue() {
        let records = classifications();
        let references = records.iter().collect::<Vec<_>>();
        let mut queue = ReflectionQueue::default();
        queue.enqueue(proposals_for_clusters(&recurring_clusters(&references)));
        assert_eq!(queue.pending().len(), 2);
    }

    #[test]
    fn nothing_auto_applies() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::BugRegression {
            check_id: "check".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        assert!(queue.pending().len() == 1);
    }

    #[test]
    fn accept_bug() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::BugRegression {
            check_id: "check".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        let accepted = queue.accept(0, ProjectId::generate()).unwrap();
        let mut regression = RegressionSet::default();
        assert!(apply_bug_acceptance(&mut regression, &accepted).unwrap());
        assert!(regression.contains("check"));
    }

    #[test]
    fn accept_spec_gap() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::SpecGapClause {
            check_id: "check".into(),
            clause: "clause".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        assert!(matches!(
            queue.accept(0, ProjectId::generate()).unwrap(),
            AcceptedCalibration::SpecGapConcept {
                event: WikiEvent::PageCreated { .. }
            }
        ));
    }

    #[test]
    fn no_fourth_tier() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::SpecGapClause {
            check_id: "check".into(),
            clause: "clause".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        let AcceptedCalibration::SpecGapConcept { event } =
            queue.accept(0, ProjectId::generate()).unwrap()
        else {
            panic!("concept")
        };
        assert!(matches!(event, WikiEvent::PageCreated { .. }));
    }

    #[test]
    fn accept_noise() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::NoiseRecalibration {
            check_id: "check".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        assert!(matches!(
            queue.accept(0, ProjectId::generate()).unwrap(),
            AcceptedCalibration::NoiseRecalibrated { .. }
        ));
    }

    #[test]
    fn accept_ambiguity() {
        let mut queue = ReflectionQueue::default();
        queue.enqueue([CalibrationProposal::AmbiguityInterviewRule {
            check_id: "check".into(),
            rule: "ask".into(),
            tasks: vec!["a".into(), "b".into()],
        }]);
        assert!(
            matches!(queue.accept(0, ProjectId::generate()).unwrap(), AcceptedCalibration::AmbiguityRule { rule } if rule == "ask")
        );
    }

    #[test]
    fn rejection_sticks() {
        let proposal = CalibrationProposal::BugRegression {
            check_id: "check".into(),
            tasks: vec!["a".into(), "b".into()],
        };
        let mut queue = ReflectionQueue::default();
        queue.enqueue([proposal.clone()]);
        queue.reject(0).unwrap();
        queue.enqueue([proposal.clone()]);
        assert!(queue.pending().is_empty());
        assert!(queue.rejected(&proposal));
    }

    #[test]
    fn threshold_gates_injection() {
        assert!(specialization_injected(
            0.8,
            default_specialization_threshold()
        ));
    }

    #[test]
    fn below_threshold_not_injected() {
        assert!(!specialization_injected(
            0.79,
            default_specialization_threshold()
        ));
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod calibration {
    use super::*;

    fn usage(input: u64, cache_read: u64) -> Usage {
        Usage {
            input: Some(input),
            output: Some(1),
            cache_read: Some(cache_read),
            cache_write: None,
        }
    }

    #[test]
    fn cache_rate_criterion() {
        assert!(super::cache_rate_criterion(
            &usage(100, 80),
            &usage(100, 80)
        ));
        assert!(!super::cache_rate_criterion(
            &usage(100, 80),
            &usage(100, 79)
        ));
        assert!(!super::cache_rate_criterion(
            &Usage::default(),
            &usage(100, 80)
        ));

        let comparison = PairedContextRuns {
            baseline: CacheRateArm::new("control", [usage(100, 80), usage(50, 40)]),
            candidate: CacheRateArm::new("context-policy", [usage(100, 90), usage(50, 45)]),
        };
        let decision = paired_cache_rate_criterion(&comparison);
        assert!(decision.is_non_regression());
        assert!(matches!(decision, CacheRateDecision::Pass { .. }));
    }
}
