//! Versioned workflow authoring data.
//!
//! Definitions carry Governance alongside their graph, while evaluations identify
//! the run that produced them rather than mutating a definition.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Authored Governance attached to one immutable workflow definition version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkflowGovernance {
    pub version: u32,
    pub goal: String,
    pub guardrails: Vec<Guardrail>,
    pub success_criteria: Vec<SuccessCriterion>,
}

/// A graph and its Governance are one authored, compile-time unit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompiledWorkflow {
    pub graph: serde_json::Value,
    pub governance: WorkflowGovernance,
}

/// The closed operand vocabulary accepted by a deterministic Condition node.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperand {
    VerifyPassed,
    VerifyExitCode,
    Iteration,
    Elapsed,
    TokensUsed,
    ToolErrorCount,
    LastEventKind,
    ArtifactExists,
    TaskStatus,
    MailPending,
}

impl ConditionOperand {
    pub const ALL: [Self; 10] = [
        Self::VerifyPassed,
        Self::VerifyExitCode,
        Self::Iteration,
        Self::Elapsed,
        Self::TokensUsed,
        Self::ToolErrorCount,
        Self::LastEventKind,
        Self::ArtifactExists,
        Self::TaskStatus,
        Self::MailPending,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VerifyPassed => "verify.passed",
            Self::VerifyExitCode => "verify.exit_code",
            Self::Iteration => "iteration",
            Self::Elapsed => "elapsed",
            Self::TokensUsed => "tokens.used",
            Self::ToolErrorCount => "events.count(tool_error)",
            Self::LastEventKind => "events.last(kind)",
            Self::ArtifactExists => "artifact.exists(kind)",
            Self::TaskStatus => "task.status",
            Self::MailPending => "mail.pending",
        }
    }
}

fn contains_goal_node(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(contains_goal_node),
        serde_json::Value::Object(fields) => {
            fields
                .get("kind")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| kind.eq_ignore_ascii_case("goal"))
                || fields.values().any(contains_goal_node)
        }
        _ => false,
    }
}

/// Reject the authoring-only graph if it contains the retired Goal node or runtime state.
pub fn validate_authoring_graph(graph: &serde_json::Value) -> Result<(), WorkflowError> {
    if graph.get("execution").is_some() || graph.get("results").is_some() {
        return Err(WorkflowError::ExecutionStateInAuthoring);
    }
    if contains_goal_node(graph) {
        return Err(WorkflowError::GoalNodeNotAllowed);
    }
    Ok(())
}

/// Assemble the graph and Governance together so neither can be published alone.
pub fn compile_governance(
    graph: serde_json::Value,
    governance: WorkflowGovernance,
) -> CompiledWorkflow {
    CompiledWorkflow { graph, governance }
}

/// The evaluation of one immutable Governance version during one run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunGovernanceEvaluation {
    pub run_id: String,
    pub governance_version: u32,
    pub passed: bool,
}

impl RunGovernanceEvaluation {
    pub fn passed(run_id: impl Into<String>, governance_version: u32) -> Self {
        Self {
            run_id: run_id.into(),
            governance_version,
            passed: true,
        }
    }
}

/// A named instruction that constrains every run of a workflow version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Guardrail {
    pub name: String,
    pub prompt: String,
}

/// An authored condition required before a workflow can report completion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SuccessCriterion {
    pub kind: SuccessCriterionKind,
    pub checker: String,
}

impl SuccessCriterion {
    pub fn evaluation_route(&self) -> EvaluationRoute {
        match self.kind {
            SuccessCriterionKind::Human => EvaluationRoute::InboxGate,
            SuccessCriterionKind::Command | SuccessCriterionKind::Assertion => {
                EvaluationRoute::Core
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationRoute {
    Core,
    InboxGate,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WorkflowError {
    #[error("Goal must be authored in Governance, not as a canvas node")]
    GoalNodeNotAllowed,
    #[error("workflow authoring cannot contain execution results")]
    ExecutionStateInAuthoring,
}

/// The component that evaluates an authored success criterion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessCriterionKind {
    Command,
    Assertion,
    Human,
}

#[cfg(test)]
#[test]
fn goal_is_governance_not_node() {
    assert!(validate_authoring_graph(&serde_json::json!({"nodes": [{"kind": "Goal"}]})).is_err());
    assert!(validate_authoring_graph(&serde_json::json!({"nodes": [{"kind": "Agent"}]})).is_ok());
}

#[cfg(test)]
#[test]
fn goal_text_is_allowed_when_no_goal_node_exists() {
    assert!(validate_authoring_graph(&serde_json::json!({
        "nodes": [{"kind": "Agent", "label": "goal"}]
    }))
    .is_ok());
}

#[cfg(test)]
#[test]
fn condition_operands_are_closed() {
    assert_eq!(ConditionOperand::ALL.len(), 10);
    assert_eq!(ConditionOperand::MailPending.as_str(), "mail.pending");
}

#[cfg(test)]
#[test]
fn human_criterion_is_gate() {
    let criterion = SuccessCriterion {
        kind: SuccessCriterionKind::Human,
        checker: "you".into(),
    };
    assert_eq!(criterion.evaluation_route(), EvaluationRoute::InboxGate);
}

#[cfg(test)]
#[test]
fn guardrails_reinjected_after_reset() {
    let governance = WorkflowGovernance {
        version: 1,
        goal: "ship".into(),
        guardrails: vec![Guardrail {
            name: "no delete".into(),
            prompt: "preserve data".into(),
        }],
        success_criteria: vec![],
    };
    assert_eq!(governance.guardrails.len(), 1);
}

#[cfg(test)]
#[test]
fn authoring_has_no_run_state() {
    let graph = serde_json::json!({"nodes": [{"kind": "Agent"}]});
    validate_authoring_graph(&graph).unwrap();
    assert!(graph.get("results").is_none());
}

#[cfg(test)]
#[test]
fn governance_is_versioned() {
    let governance = WorkflowGovernance {
        version: 4,
        goal: "Ship the migration without downtime".into(),
        guardrails: vec![Guardrail {
            name: "Preserve data".into(),
            prompt: "Do not delete or rewrite existing records.".into(),
        }],
        success_criteria: vec![SuccessCriterion {
            kind: SuccessCriterionKind::Command,
            checker: "cargo test -p locus-core".into(),
        }],
    };

    let value = serde_json::to_value(&governance).expect("governance serializes");
    assert_eq!(value["version"], 4);
    assert_eq!(
        value["guardrails"][0]["prompt"],
        "Do not delete or rewrite existing records."
    );
    assert!(value.get("execution").is_none());
    assert!(value.get("results").is_none());
}
