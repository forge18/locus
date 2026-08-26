mod arbiter {
    use locus_core::services::arbiter::{
        ambiguity_rate, arbitrate, classify, spec_gap_rate, ArbiterAction, FailureClass,
        FailureEvidence, Iteration, RegressionSet,
    };

    fn evidence() -> FailureEvidence {
        FailureEvidence::new("unit-tests", 1)
    }

    #[test]
    fn classifies() {
        assert_eq!(classify(&evidence()).unwrap(), FailureClass::Bug);
        let mut noise = evidence();
        noise.check_is_flaky = true;
        assert_eq!(classify(&noise).unwrap(), FailureClass::Noise);
    }

    #[test]
    fn column_on_iteration() {
        let mut iteration = Iteration::new(1).unwrap();
        let mut regression = RegressionSet::default();
        arbitrate(&evidence(), &mut iteration, &mut regression, None, None).unwrap();
        assert_eq!(iteration.arbiter_class, Some(FailureClass::Bug));
        assert!(iteration.counts_toward_budget);
    }

    #[test]
    fn bug_promotes_check() {
        let mut iteration = Iteration::new(1).unwrap();
        let mut regression = RegressionSet::default();
        let decision = arbitrate(&evidence(), &mut iteration, &mut regression, None, None).unwrap();
        assert!(matches!(
            decision.action,
            ArbiterAction::BugRetry { promoted: true, .. }
        ));
        assert!(regression.contains("unit-tests"));
    }

    #[test]
    fn noise_is_free() {
        let mut noise = evidence();
        noise.check_is_flaky = true;
        let mut iteration = Iteration::new(1).unwrap();
        let mut regression = RegressionSet::default();
        let decision = arbitrate(&noise, &mut iteration, &mut regression, None, None).unwrap();
        assert_eq!(decision.class, FailureClass::Noise);
        assert!(!decision.counts_toward_budget);
        assert!(!iteration.counts_toward_budget);
    }

    #[test]
    fn spec_gap_exits() {
        let mut gap = evidence();
        gap.requirement_missing = true;
        let mut iteration = Iteration::new(1).unwrap();
        let mut regression = RegressionSet::default();
        let decision = arbitrate(
            &gap,
            &mut iteration,
            &mut regression,
            Some("Document the missing requirement"),
            None,
        )
        .unwrap();
        assert!(decision.action.exits_workflow());
    }

    #[test]
    fn ambiguity_restarts() {
        let mut ambiguity = evidence();
        ambiguity.requirement_ambiguous = true;
        let mut iteration = Iteration::new(1).unwrap();
        let mut regression = RegressionSet::default();
        let decision = arbitrate(
            &ambiguity,
            &mut iteration,
            &mut regression,
            None,
            Some("The requirement now has one reading"),
        )
        .unwrap();
        assert!(decision.action.restarts_implementation());
        assert!(!matches!(decision.action, ArbiterAction::BugRetry { .. }));
    }

    #[test]
    fn rates_are_queries() {
        let mut bug = Iteration::new(1).unwrap();
        bug.record_failure(FailureClass::Bug);
        let mut gap = Iteration::new(2).unwrap();
        gap.record_failure(FailureClass::SpecGap);
        let mut ambiguity = Iteration::new(3).unwrap();
        ambiguity.record_failure(FailureClass::Ambiguity);
        assert_eq!(
            spec_gap_rate(&[bug.clone(), gap.clone(), ambiguity.clone()]),
            33
        );
        assert_eq!(ambiguity_rate(&[bug, gap, ambiguity]), 33);
    }
}
