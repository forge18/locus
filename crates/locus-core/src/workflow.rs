//! Versioned workflow authoring data.
//!
//! Definitions carry Governance alongside their graph, while execution data belongs
//! to run records. This module deliberately defines no execution or result types.

use serde::{Deserialize, Serialize};

/// Authored Governance attached to one immutable workflow definition version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkflowGovernance {
    pub version: u32,
    pub goal: String,
    pub guardrails: Vec<Guardrail>,
    pub success_criteria: Vec<SuccessCriterion>,
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
