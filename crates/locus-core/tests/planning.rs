mod planning {
    use locus_core::planning::{EditableSpec, PlanningStage, Requirement};

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
