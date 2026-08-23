mod workflow {
    use locus_core::workflow::{Guardrail, SuccessCriterion, SuccessCriterionKind, WorkflowGovernance};

    #[test]
    fn governance_compiles() {
        let governance = WorkflowGovernance { version: 1, goal: "Ship".into(), guardrails: vec![], success_criteria: vec![] };
        let compiled = locus_core::workflow::compile_governance(serde_json::json!({"nodes": []}), governance.clone());
        assert_eq!(compiled.governance, governance);
        assert_eq!(compiled.graph["nodes"], serde_json::json!([]));
    }

    #[test]
    fn success_criteria() {
        let criterion = SuccessCriterion { kind: SuccessCriterionKind::Command, checker: "cargo test".into() };
        assert_eq!(criterion.kind, SuccessCriterionKind::Command);
        assert_eq!(criterion.checker, "cargo test");
    }

    #[test]
    fn guardrail_prompts() {
        let guardrail = Guardrail { name: "safe".into(), prompt: "preserve data".into() };
        assert_eq!((guardrail.name.as_str(), guardrail.prompt.as_str()), ("safe", "preserve data"));
    }

    #[test]
    fn goal_not_node() {
        let governance = WorkflowGovernance { version: 1, goal: "Ship".into(), guardrails: vec![], success_criteria: vec![] };
        assert_eq!(governance.goal, "Ship");
    }

    #[test]
    fn governance_root() {
        let governance = WorkflowGovernance { version: 1, goal: "Ship".into(), guardrails: vec![Guardrail { name: "safe".into(), prompt: "preserve".into() }], success_criteria: vec![SuccessCriterion { kind: SuccessCriterionKind::Command, checker: "cargo test".into() }] };
        assert_eq!(governance.version, 1);
    }
}
