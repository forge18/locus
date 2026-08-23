mod planning {
    use locus_core::planning::{ApprovedPlan, CardMode, Decomposition, EditableSpec, PlanTask, PlanningStage, Requirement};

    #[test]
    fn spec_only_cards() {
        let plan = ApprovedPlan::new("Spec", vec![PlanTask::new("T-01", "One")]);
        let cards = Decomposition::for_spec_only(plan);

        assert_eq!(cards.cards().len(), 1);
    }

    #[test]
    fn task_decomposition() {
        let task = PlanTask::new("T-01", "Implement")
            .with_role("builder")
            .with_estimate_minutes(45)
            .with_dependencies(["T-00"]);

        assert_eq!(task.role, "builder");
        assert_eq!(task.estimate_minutes, 45);
        assert_eq!(CardMode::SelectedCarveOuts.card_count(3), 4);
    }

    #[test]
    fn reaudits_changed_requirements() {
        let mut spec = EditableSpec::new([Requirement::new("R-01", "one"), Requirement::new("R-02", "two")]).expect("spec");
        spec.edit("R-02", "changed").expect("edit");

        assert_eq!(spec.changed_requirements().map(|requirement| requirement.id.as_str()).collect::<Vec<_>>(), ["R-02"]);
        spec.mark_reaudited();
        assert!(spec.changed_requirements().next().is_none());
    }

    #[test]
    fn editable_requirements() {
        let mut spec = EditableSpec::new([Requirement::new("R-01", "Keep branches")]).expect("spec");
        spec.edit("R-01", "Keep every branch").expect("edit");

        assert_eq!(spec.requirement("R-01").expect("requirement").body, "Keep every branch");
    }

    #[test]
    fn nine_stages() {
        assert_eq!(PlanningStage::ALL.len(), 9);
        assert_eq!(PlanningStage::Inputs.next(), Some(PlanningStage::Orient));
        assert_eq!(PlanningStage::Approve.next(), None);
    }
}
