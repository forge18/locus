mod workflow {
    use locus_core::workflow::{Guardrail, SuccessCriterion, SuccessCriterionKind, WorkflowGovernance};

    #[test]
    fn governance_root() {
        let governance = WorkflowGovernance { version: 1, goal: "Ship".into(), guardrails: vec![Guardrail { name: "safe".into(), prompt: "preserve".into() }], success_criteria: vec![SuccessCriterion { kind: SuccessCriterionKind::Command, checker: "cargo test".into() }] };
        assert_eq!(governance.version, 1);
    }
}
