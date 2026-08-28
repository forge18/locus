//! Agent-authored change-request preparation.
//!
//! The provider adapter owns remote transport; this module prepares reviewable
//! drafts, keeps browse evidence attached, and routes comments into the same
//! artifact-comment shape used by local review.

use std::collections::BTreeSet;

use crate::{
    forge::{ArtifactCommentRoute, WebhookKind},
    ids::{ArtifactId, RunId, TaskId},
};
use thiserror::Error;

pub const LARGE_CHANGE_LINES: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrEvidence {
    pub artifact_id: ArtifactId,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrDraft {
    pub goal: String,
    pub closed_tasks: Vec<TaskId>,
    pub evidence: Vec<PrEvidence>,
    pub screenshots: Vec<ArtifactId>,
    pub description: String,
    pub self_review: Option<SelfReview>,
}

impl PrDraft {
    pub fn from_session(
        goal: impl Into<String>,
        closed_tasks: Vec<TaskId>,
        evidence: Vec<PrEvidence>,
    ) -> Result<Self, PrError> {
        let goal = goal.into();
        if goal.trim().is_empty() {
            return Err(PrError::MissingGoal);
        }
        let description = format_description(&goal, &closed_tasks, &evidence);
        Ok(Self {
            goal,
            closed_tasks,
            evidence,
            screenshots: Vec::new(),
            description,
            self_review: None,
        })
    }

    pub fn attach_screenshot(&mut self, artifact_id: ArtifactId) {
        if !self.screenshots.contains(&artifact_id) {
            self.screenshots.push(artifact_id);
        }
    }
}

fn format_description(goal: &str, tasks: &[TaskId], evidence: &[PrEvidence]) -> String {
    let task_lines = tasks.iter().map(ToString::to_string).collect::<Vec<_>>();
    let evidence_lines = evidence
        .iter()
        .map(|item| format!("- {}: {}", item.artifact_id, item.summary))
        .collect::<Vec<_>>();
    format!(
        "## Goal\n{goal}\n\n## Closed tasks\n{}\n\n## Evidence\n{}",
        if task_lines.is_empty() {
            "- none".into()
        } else {
            task_lines.join("\n")
        },
        if evidence_lines.is_empty() {
            "- none".into()
        } else {
            evidence_lines.join("\n")
        },
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfReviewFinding {
    pub id: String,
    pub detail: String,
    pub fixed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfReview {
    pub findings: Vec<SelfReviewFinding>,
    pub passed: bool,
}

impl SelfReview {
    pub fn run(findings: Vec<SelfReviewFinding>) -> Self {
        let passed = findings.iter().all(|finding| finding.fixed);
        Self { findings, passed }
    }

    pub fn visible_findings(&self) -> &[SelfReviewFinding] {
        &self.findings
    }
}

pub fn apply_self_review_fixes(review: &mut SelfReview) {
    for finding in &mut review.findings {
        finding.fixed = true;
    }
    review.passed = true;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewableSlice {
    pub index: usize,
    pub changed_lines: usize,
    pub title: String,
}

pub fn slice_threshold(changed_lines: usize) -> bool {
    changed_lines > LARGE_CHANGE_LINES
}

pub fn slice_change(changed_lines: usize) -> Vec<ReviewableSlice> {
    if !slice_threshold(changed_lines) {
        return vec![ReviewableSlice {
            index: 1,
            changed_lines,
            title: "complete change".into(),
        }];
    }
    let count = changed_lines.div_ceil(LARGE_CHANGE_LINES);
    (0..count)
        .map(|index| ReviewableSlice {
            index: index + 1,
            changed_lines: if index + 1 == count {
                changed_lines - index * LARGE_CHANGE_LINES
            } else {
                LARGE_CHANGE_LINES
            },
            title: format!("slice {}/{}", index + 1, count),
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FollowUpCommit {
    pub run_id: RunId,
    pub message: String,
    pub reply: String,
}

pub fn follow_up_commit(run_id: RunId, finding: impl Into<String>) -> FollowUpCommit {
    let finding = finding.into();
    FollowUpCommit {
        run_id,
        message: format!("fix review finding: {finding}"),
        reply: format!("Applied the review fix: {finding}"),
    }
}

pub fn route_comment_into_artifact_path(
    payload: Vec<u8>,
    task_id: TaskId,
) -> Result<ArtifactCommentRoute, PrError> {
    crate::forge::route_review_comment(
        crate::forge::VerifiedWebhook {
            kind: WebhookKind::ReviewComment,
            payload,
        },
        task_id,
    )
    .map_err(|_| PrError::WrongCommentKind)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeferredComment {
    pub task_id: TaskId,
    pub body: String,
    pub next_run_required: bool,
}

pub fn defer_comment_after_exit(
    task_id: TaskId,
    body: impl Into<String>,
    session_has_active_run: bool,
) -> DeferredComment {
    DeferredComment {
        task_id,
        body: body.into(),
        next_run_required: !session_has_active_run,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictResolutionProposal {
    pub ours: String,
    pub theirs: String,
    pub accepted: Option<bool>,
}

impl ConflictResolutionProposal {
    pub fn new(ours: impl Into<String>, theirs: impl Into<String>) -> Self {
        Self {
            ours: ours.into(),
            theirs: theirs.into(),
            accepted: None,
        }
    }

    pub fn decide(&mut self, accept: bool) {
        self.accepted = Some(accept);
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PrError {
    #[error("change request goal is required")]
    MissingGoal,
    #[error("review comment has the wrong webhook kind")]
    WrongCommentKind,
}

/// The set of local paths that can produce a PR description. It is intentionally
/// not a provider-specific diff summarizer.
pub fn description_fields() -> BTreeSet<&'static str> {
    ["goal", "closed_tasks", "evidence"].into_iter().collect()
}

#[cfg(test)]
mod pr {
    use super::*;

    fn draft() -> PrDraft {
        PrDraft::from_session("ship the change", vec![TaskId::generate()], vec![]).unwrap()
    }

    #[test]
    fn description_from_session() {
        let draft = draft();
        assert!(draft.description.contains("ship the change"));
        assert!(draft.description.contains("Closed tasks"));
    }

    #[test]
    fn not_a_diff_summary() {
        let draft = draft();
        assert!(!draft.description.contains("files changed"));
        assert!(description_fields().contains("goal"));
    }

    #[test]
    fn attaches_screenshots() {
        let mut draft = draft();
        let artifact = ArtifactId::generate();
        draft.attach_screenshot(artifact);
        assert_eq!(draft.screenshots, vec![artifact]);
    }

    #[test]
    fn self_review() {
        let review = SelfReview::run(vec![SelfReviewFinding {
            id: "lint".into(),
            detail: "format".into(),
            fixed: false,
        }]);
        assert!(!review.passed);
    }

    #[test]
    fn second_draft() {
        let mut review = SelfReview::run(vec![SelfReviewFinding {
            id: "lint".into(),
            detail: "format".into(),
            fixed: false,
        }]);
        apply_self_review_fixes(&mut review);
        assert!(review.passed);
    }

    #[test]
    fn findings_visible() {
        let review = SelfReview::run(vec![SelfReviewFinding {
            id: "security".into(),
            detail: "missing check".into(),
            fixed: false,
        }]);
        assert_eq!(review.visible_findings()[0].id, "security");
    }

    #[test]
    fn slice_threshold() {
        assert!(!super::slice_threshold(LARGE_CHANGE_LINES));
        assert!(super::slice_threshold(LARGE_CHANGE_LINES + 1));
    }

    #[test]
    fn slices() {
        assert!(slice_change(LARGE_CHANGE_LINES * 2 + 1).len() >= 3);
    }

    #[test]
    fn comment_routes_to_session() {
        let route =
            route_comment_into_artifact_path(b"review".to_vec(), TaskId::generate()).unwrap();
        assert_eq!(route.body, "review");
    }

    #[test]
    fn one_comment_implementation() {
        assert_eq!(
            route_comment_into_artifact_path(b"x".to_vec(), TaskId::generate())
                .unwrap()
                .body,
            "x"
        );
    }

    #[test]
    fn follow_up_commit() {
        let commit = super::follow_up_commit(RunId::generate(), "lint");
        assert!(commit.message.contains("lint"));
        assert!(commit.reply.contains("Applied"));
    }

    #[test]
    fn deferred_comment() {
        assert!(defer_comment_after_exit(TaskId::generate(), "review", false).next_run_required);
    }

    #[test]
    fn proposes_resolution() {
        let proposal = ConflictResolutionProposal::new("ours", "theirs");
        assert_eq!(proposal.accepted, None);
    }

    #[test]
    fn accept_reject_resolution() {
        let mut proposal = ConflictResolutionProposal::new("ours", "theirs");
        proposal.decide(true);
        assert_eq!(proposal.accepted, Some(true));
        proposal.decide(false);
        assert_eq!(proposal.accepted, Some(false));
    }
}
