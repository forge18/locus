mod planning {
    use locus_core::services::planning::{
        ApprovedPlan, CardMode, Decomposition, EditableSpec, PlanTask, PlanningStage, Requirement,
    };

    #[test]
    fn cards_keep_dependencies() {
        let plan = ApprovedPlan::new(
            "Spec",
            vec![
                PlanTask::new("T-01", "One"),
                PlanTask::new("T-02", "Two").with_dependencies(["T-01"]),
            ],
        );
        let cards = Decomposition::every_task(plan)
            .expect("mapping")
            .approve()
            .expect("approve");

        assert_eq!(cards[1].dependencies, ["task:T-01"]);
    }

    #[test]
    fn approval_commits_mapping() {
        let plan = ApprovedPlan::new("Spec", vec![PlanTask::new("T-01", "One")]);
        let cards = Decomposition::every_task(plan)
            .expect("mapping")
            .approve()
            .expect("approve");

        assert_eq!(cards.len(), 1);
    }

    #[test]
    fn carve_out_cards() {
        let plan = ApprovedPlan::new(
            "Spec",
            vec![PlanTask::new("T-01", "One"), PlanTask::new("T-02", "Two")],
        );
        let cards = Decomposition::spec_plus_selected(plan, ["T-02"]).expect("cards");

        assert_eq!(cards.cards().len(), 2);
    }

    #[test]
    fn every_task_cards() {
        let plan = ApprovedPlan::new(
            "Spec",
            vec![PlanTask::new("T-01", "One"), PlanTask::new("T-02", "Two")],
        );
        let cards = Decomposition::every_task(plan).expect("cards");

        assert_eq!(cards.cards().len(), 2);
    }

    #[test]
    fn spec_only_cards() {
        let plan = ApprovedPlan::new("Spec", vec![PlanTask::new("T-01", "One")]);
        let cards = Decomposition::spec_only(plan);

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
        let mut spec = EditableSpec::new([
            Requirement::new("R-01", "one").expect("valid requirement"),
            Requirement::new("R-02", "two").expect("valid requirement"),
        ])
        .expect("spec");
        spec.edit("R-02", "changed").expect("edit");

        assert_eq!(
            spec.changed_requirements()
                .map(Requirement::id)
                .collect::<Vec<_>>(),
            ["R-02"]
        );
        spec.mark_reaudited();
        assert!(spec.changed_requirements().next().is_none());
    }

    #[test]
    fn editable_requirements() {
        let mut spec = EditableSpec::new([
            Requirement::new("R-01", "Keep branches").expect("valid requirement")
        ])
        .expect("spec");
        spec.edit("R-01", "Keep every branch").expect("edit");

        assert_eq!(
            spec.requirement("R-01").expect("requirement").body(),
            "Keep every branch"
        );
    }

    #[test]
    fn requirements_cannot_bypass_nonempty_validation() {
        assert!(Requirement::new("", "body").is_err());
        assert!(Requirement::new("R-01", " ").is_err());
    }

    #[test]
    fn nine_stages() {
        assert_eq!(PlanningStage::ALL.len(), 9);
        assert_eq!(PlanningStage::Inputs.next(), Some(PlanningStage::Orient));
        assert_eq!(PlanningStage::Approve.next(), None);
    }
}
