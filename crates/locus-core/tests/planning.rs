mod planning {
    use locus_core::planning::{EditableSpec, PlanningStage, Requirement};

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
