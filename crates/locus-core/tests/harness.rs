mod harness {
    use locus_core::{
        harness::selection::{HarnessDescriptor, HarnessSelectionError, ProjectHarnessPolicy},
        runtime::routing::RoutingDefaults,
    };
    use std::collections::BTreeSet;

    #[test]
    fn defaults() {
        let defaults = RoutingDefaults {
            model_id: "opus".into(),
            effort: "high".into(),
        };
        let harness = HarnessDescriptor {
            identifier: "claude".into(),
            adapter: None,
            compatible_providers: BTreeSet::new(),
            defaults: defaults.clone(),
        };
        assert_eq!(harness.defaults, defaults);
    }

    #[test]
    fn project_provider_gate() {
        let policy = ProjectHarnessPolicy {
            permitted_harnesses: BTreeSet::from(["claude".into()]),
            configured_providers: BTreeSet::new(),
        };
        let harness = HarnessDescriptor {
            identifier: "claude".into(),
            adapter: Some(locus_core::harness::selection::HarnessAdapter {
                identity: "acp".into(),
            }),
            compatible_providers: BTreeSet::from(["anthropic".into()]),
            defaults: RoutingDefaults {
                model_id: "sonnet".into(),
                effort: "medium".into(),
            },
        };
        assert!(matches!(
            policy.select(&harness, "anthropic"),
            Err(HarnessSelectionError::ProviderNotConfigured(_))
        ));
    }

    #[test]
    fn provider_compatibility() {
        let policy = ProjectHarnessPolicy {
            permitted_harnesses: BTreeSet::from(["claude".into()]),
            configured_providers: BTreeSet::from(["openai".into()]),
        };
        let harness = HarnessDescriptor {
            identifier: "claude".into(),
            adapter: Some(locus_core::harness::selection::HarnessAdapter {
                identity: "acp".into(),
            }),
            compatible_providers: BTreeSet::from(["anthropic".into()]),
            defaults: RoutingDefaults {
                model_id: "sonnet".into(),
                effort: "medium".into(),
            },
        };
        assert!(matches!(
            policy.select(&harness, "openai"),
            Err(HarnessSelectionError::ProviderIncompatible { .. })
        ));
    }

    #[test]
    fn adapter_gate() {
        let policy = ProjectHarnessPolicy {
            permitted_harnesses: BTreeSet::from(["aider".into()]),
            configured_providers: BTreeSet::from(["anthropic".into()]),
        };
        let harness = HarnessDescriptor {
            identifier: "aider".into(),
            adapter: None,
            compatible_providers: BTreeSet::from(["anthropic".into()]),
            defaults: RoutingDefaults {
                model_id: "sonnet".into(),
                effort: "medium".into(),
            },
        };
        assert_eq!(
            policy.select(&harness, "anthropic"),
            Err(HarnessSelectionError::AdapterUnavailable("aider".into()))
        );
    }
}
