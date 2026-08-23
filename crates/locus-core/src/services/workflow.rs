//! Versioned workflow authoring data.
//!
//! Definitions carry Governance alongside their graph, while evaluations identify
//! the run that produced them rather than mutating a definition.

use serde::{Deserialize, Serialize};

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
