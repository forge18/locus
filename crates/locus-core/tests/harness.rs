mod harness {
    use std::collections::BTreeSet;
    use locus_core::{harness::{HarnessDescriptor, HarnessSelectionError, ProjectHarnessPolicy}, routing::RoutingDefaults};

    #[test]
    fn adapter_gate() {
        let policy = ProjectHarnessPolicy { permitted_harnesses: BTreeSet::from(["aider".into()]), configured_providers: BTreeSet::from(["anthropic".into()]) };
        let harness = HarnessDescriptor { identifier: "aider".into(), adapter: None, compatible_providers: BTreeSet::from(["anthropic".into()]), defaults: RoutingDefaults { model_id: "sonnet".into(), effort: "medium".into() } };
        assert_eq!(policy.select(&harness, "anthropic"), Err(HarnessSelectionError::AdapterUnavailable("aider".into())));
    }
}
