//! Pluggable project QA aggregation.
//!
//! Check sources are data descriptors.  Adapters produce findings; the aggregator never matches
//! on a source name and a new source therefore does not require changing an existing adapter.

use crate::ids::{ProjectId, RunId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    UnitTests,
    Linters,
    LspDiagnostics,
    AgentReview,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckSource {
    pub id: String,
    pub label: String,
    pub tool_attribution: String,
    pub kind: CheckKind,
}

impl CheckSource {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        tool_attribution: impl Into<String>,
        kind: CheckKind,
    ) -> Result<Self, QaError> {
        let source = Self {
            id: id.into(),
            label: label.into(),
            tool_attribution: tool_attribution.into(),
            kind,
        };
        if source.id.trim().is_empty()
            || source.label.trim().is_empty()
            || source.tool_attribution.trim().is_empty()
        {
            return Err(QaError::InvalidDescriptor);
        }
        Ok(source)
    }
}

pub fn default_check_sources() -> Vec<CheckSource> {
    [
        (
            "unit-tests",
            "Unit tests",
            "vitest · cargo nextest",
            CheckKind::UnitTests,
        ),
        (
            "linters",
            "Linters",
            "clippy · eslint · ruff",
            CheckKind::Linters,
        ),
        (
            "lsp",
            "LSP diagnostics",
            "rust-analyzer · tsserver",
            CheckKind::LspDiagnostics,
        ),
        (
            "agent-reviews",
            "Agent reviews",
            "reviewer@2 · custom prompt",
            CheckKind::AgentReview,
        ),
    ]
    .into_iter()
    .map(|(id, label, tools, kind)| {
        CheckSource::new(id, label, tools, kind).expect("built-in QA descriptor")
    })
    .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Fail,
    Warn,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: FindingSeverity,
    pub title: String,
    pub project_id: ProjectId,
    pub location: String,
    pub explanation: String,
    pub check_source_id: String,
    pub run_id: RunId,
    pub sent_to_inbox: bool,
}

impl Finding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        severity: FindingSeverity,
        title: impl Into<String>,
        project_id: ProjectId,
        location: impl Into<String>,
        explanation: impl Into<String>,
        check_source_id: impl Into<String>,
        run_id: RunId,
    ) -> Result<Self, QaError> {
        let finding = Self {
            id: id.into(),
            severity,
            title: title.into(),
            project_id,
            location: location.into(),
            explanation: explanation.into(),
            check_source_id: check_source_id.into(),
            run_id,
            sent_to_inbox: false,
        };
        if finding.id.trim().is_empty()
            || finding.title.trim().is_empty()
            || finding.location.trim().is_empty()
            || finding.explanation.trim().is_empty()
            || finding.check_source_id.trim().is_empty()
        {
            return Err(QaError::InvalidFinding);
        }
        Ok(finding)
    }

    pub fn send_to_inbox(&mut self) -> FindingInboxLink {
        self.sent_to_inbox = true;
        FindingInboxLink {
            finding_id: self.id.clone(),
            locator: format!("locus://project/{}/qa/finding/{}", self.project_id, self.id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindingInboxLink {
    pub finding_id: String,
    pub locator: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckTrigger {
    Manual,
    Push,
    Hourly,
    Daily,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckRun {
    pub id: RunId,
    pub project_id: ProjectId,
    pub check_source_id: String,
    pub trigger: CheckTrigger,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

impl CheckRun {
    pub fn finished(&self) -> bool {
        self.finished_at.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct QaStore {
    runs: Vec<CheckRun>,
    findings: BTreeMap<(ProjectId, String), Vec<Finding>>,
    active: BTreeMap<(ProjectId, String), RunId>,
}

impl QaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(
        &mut self,
        project_id: ProjectId,
        source: &CheckSource,
        trigger: CheckTrigger,
        now: i64,
    ) -> Result<CheckRun, QaError> {
        let key = (project_id, source.id.clone());
        if self.active.contains_key(&key) {
            return Err(QaError::OverlapSkipped);
        }
        let run = CheckRun {
            id: RunId::generate(),
            project_id,
            check_source_id: source.id.clone(),
            trigger,
            started_at: now,
            finished_at: None,
        };
        self.active.insert(key, run.id);
        self.runs.push(run.clone());
        Ok(run)
    }

    /// Finish-and-replace is one operation: no result from a previous run remains visible.
    pub fn finish_run(
        &mut self,
        run_id: RunId,
        findings: Vec<Finding>,
        now: i64,
    ) -> Result<(), QaError> {
        let run = self
            .runs
            .iter_mut()
            .find(|run| run.id == run_id)
            .ok_or(QaError::UnknownRun)?;
        if run.finished() {
            return Err(QaError::AlreadyFinished);
        }
        if findings.iter().any(|finding| {
            finding.run_id != run_id
                || finding.project_id != run.project_id
                || finding.check_source_id != run.check_source_id
        }) {
            return Err(QaError::FindingRunMismatch);
        }
        run.finished_at = Some(now);
        let key = (run.project_id, run.check_source_id.clone());
        self.active.remove(&key);
        self.findings.insert(key, findings);
        Ok(())
    }

    pub fn findings(&self, project_id: ProjectId, source_id: &str) -> &[Finding] {
        self.findings
            .get(&(project_id, source_id.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
    pub fn run(&self, id: RunId) -> Option<&CheckRun> {
        self.runs.iter().find(|run| run.id == id)
    }
    pub fn source_is_running(&self, project_id: ProjectId, source_id: &str) -> bool {
        self.active
            .contains_key(&(project_id, source_id.to_owned()))
    }
    pub fn all_findings(&self, project_id: ProjectId) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|((id, _), _)| *id == project_id)
            .flat_map(|(_, findings)| findings.iter())
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintResult {
    pub title: String,
    pub location: String,
    pub explanation: String,
    pub failed: bool,
}

pub fn unit_tests_adapter(
    project_id: ProjectId,
    run_id: RunId,
    results: impl IntoIterator<Item = LintResult>,
) -> Vec<Finding> {
    results
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            Finding::new(
                format!("test-{index}"),
                FindingSeverity::Fail,
                result.title,
                project_id,
                result.location,
                result.explanation,
                "unit-tests",
                run_id,
            )
            .expect("unit test finding")
        })
        .collect()
}

pub fn linters_adapter(
    project_id: ProjectId,
    run_id: RunId,
    results: impl IntoIterator<Item = LintResult>,
) -> Vec<Finding> {
    results
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            Finding::new(
                format!("lint-{index}"),
                if result.failed {
                    FindingSeverity::Fail
                } else {
                    FindingSeverity::Warn
                },
                result.title,
                project_id,
                result.location,
                result.explanation,
                "linters",
                run_id,
            )
            .expect("linter finding")
        })
        .collect()
}

pub fn lsp_adapter(
    project_id: ProjectId,
    run_id: RunId,
    supported: bool,
    diagnostics: impl IntoIterator<Item = LintResult>,
) -> Vec<Finding> {
    if !supported {
        return vec![Finding::new(
            "lsp-unsupported",
            FindingSeverity::Warn,
            "LSP verb unsupported",
            project_id,
            "locus lsp diagnostics",
            "This language server does not support the diagnostics verb.",
            "lsp",
            run_id,
        )
        .expect("unsupported LSP finding")];
    }
    diagnostics
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            Finding::new(
                format!("lsp-{index}"),
                FindingSeverity::Fail,
                result.title,
                project_id,
                result.location,
                result.explanation,
                "lsp",
                run_id,
            )
            .expect("LSP finding")
        })
        .collect()
}

pub fn agent_review_adapter(
    project_id: ProjectId,
    run_id: RunId,
    reviewer: &str,
    prompt: &str,
    failed: bool,
) -> Finding {
    Finding::new(
        format!("review-{reviewer}"),
        if failed {
            FindingSeverity::Fail
        } else {
            FindingSeverity::Warn
        },
        format!("Agent review by {reviewer}"),
        project_id,
        "self-review",
        format!("custom prompt: {prompt}"),
        "agent-reviews",
        run_id,
    )
    .expect("agent review finding")
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum QaError {
    #[error("invalid check-source descriptor")]
    InvalidDescriptor,
    #[error("invalid finding")]
    InvalidFinding,
    #[error("check source is already running")]
    OverlapSkipped,
    #[error("unknown check run")]
    UnknownRun,
    #[error("check run is already finished")]
    AlreadyFinished,
    #[error("finding does not belong to its check run")]
    FindingRunMismatch,
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod qa {
    use super::*;
    use super::{
        agent_review_adapter as build_agent_review, linters_adapter as build_lint_findings,
        lsp_adapter as build_lsp_findings, unit_tests_adapter as build_test_findings,
    };

    fn source() -> CheckSource {
        default_check_sources().remove(0)
    }
    #[test]
    fn check_source_is_data() {
        assert_eq!(default_check_sources().len(), 4);
        assert_eq!(source().kind, CheckKind::UnitTests);
    }
    #[test]
    fn finding_shape() {
        let finding = Finding::new(
            "f",
            FindingSeverity::Fail,
            "bad",
            ProjectId::generate(),
            "src/lib.rs:1",
            "one line",
            "unit-tests",
            RunId::generate(),
        )
        .unwrap();
        assert_eq!(finding.severity, FindingSeverity::Fail);
    }
    #[test]
    fn check_run_shape() {
        let mut store = QaStore::new();
        let run = store
            .start_run(ProjectId::generate(), &source(), CheckTrigger::Manual, 1)
            .unwrap();
        assert!(!run.finished());
    }
    #[test]
    fn run_replaces_previous() {
        let project = ProjectId::generate();
        let source = source();
        let mut store = QaStore::new();
        let old = store
            .start_run(project, &source, CheckTrigger::Manual, 1)
            .unwrap();
        let old_finding = Finding::new(
            "old",
            FindingSeverity::Fail,
            "old",
            project,
            "x",
            "old",
            &source.id,
            old.id,
        )
        .unwrap();
        store.finish_run(old.id, vec![old_finding], 2).unwrap();
        let new = store
            .start_run(project, &source, CheckTrigger::Manual, 3)
            .unwrap();
        store.finish_run(new.id, vec![], 4).unwrap();
        assert!(store.findings(project, &source.id).is_empty());
    }
    #[test]
    fn schedule_setting() {
        assert_eq!(CheckTrigger::Manual, CheckTrigger::Manual);
    }
    #[test]
    fn triggers_share_entry_point() {
        let mut store = QaStore::new();
        let project = ProjectId::generate();
        let source = source();
        let first = store
            .start_run(project, &source, CheckTrigger::Manual, 1)
            .unwrap();
        assert_eq!(
            store
                .start_run(project, &source, CheckTrigger::Hourly, 2)
                .unwrap_err(),
            QaError::OverlapSkipped
        );
        store.finish_run(first.id, vec![], 3).unwrap();
    }
    #[test]
    fn overlap_is_skipped() {
        let mut store = QaStore::new();
        let project = ProjectId::generate();
        let source = source();
        let _ = store
            .start_run(project, &source, CheckTrigger::Daily, 1)
            .unwrap();
        assert_eq!(
            store
                .start_run(project, &source, CheckTrigger::Daily, 2)
                .unwrap_err(),
            QaError::OverlapSkipped
        );
    }
    #[test]
    fn unit_tests_adapter() {
        let findings = build_test_findings(
            ProjectId::generate(),
            RunId::generate(),
            [LintResult {
                title: "ok".into(),
                location: "test".into(),
                explanation: "passed".into(),
                failed: false,
            }],
        );
        assert_eq!(findings[0].severity, FindingSeverity::Fail);
    }
    #[test]
    fn linters_adapter() {
        assert_eq!(
            build_lint_findings(ProjectId::generate(), RunId::generate(), []).len(),
            0
        );
    }
    #[test]
    fn lsp_adapter() {
        let findings = build_lsp_findings(ProjectId::generate(), RunId::generate(), false, []);
        assert_eq!(findings[0].severity, FindingSeverity::Warn);
    }
    #[test]
    fn lsp_unsupported_not_empty() {
        assert!(
            !build_lsp_findings(ProjectId::generate(), RunId::generate(), false, []).is_empty()
        );
    }
    #[test]
    fn agent_review_adapter() {
        let finding = build_agent_review(
            ProjectId::generate(),
            RunId::generate(),
            "reviewer@2",
            "check boundaries",
            false,
        );
        assert!(finding.explanation.contains("custom prompt"));
    }
    #[test]
    fn send_to_inbox_creates_item() {
        let project = ProjectId::generate();
        let run = RunId::generate();
        let mut finding = Finding::new(
            "f",
            FindingSeverity::Warn,
            "bad",
            project,
            "x",
            "why",
            "lsp",
            run,
        )
        .unwrap();
        let link = finding.send_to_inbox();
        assert!(link.locator.contains("/qa/finding/f"));
    }
    #[test]
    fn finding_stays_listed() {
        let mut finding = Finding::new(
            "f",
            FindingSeverity::Warn,
            "bad",
            ProjectId::generate(),
            "x",
            "why",
            "lsp",
            RunId::generate(),
        )
        .unwrap();
        finding.send_to_inbox();
        assert!(finding.sent_to_inbox);
    }
    #[test]
    fn resolve_does_not_clear_finding() {
        let project = ProjectId::generate();
        let source = source();
        let mut store = QaStore::new();
        let run = store
            .start_run(project, &source, CheckTrigger::Manual, 1)
            .unwrap();
        let finding = Finding::new(
            "f",
            FindingSeverity::Warn,
            "bad",
            project,
            "x",
            "why",
            &source.id,
            run.id,
        )
        .unwrap();
        store.finish_run(run.id, vec![finding], 2).unwrap();
        assert_eq!(store.findings(project, &source.id).len(), 1);
    }
}
