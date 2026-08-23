mod planning {
    use locus_core::planning::PlanningStage;

    #[test]
    fn nine_stages() {
        assert_eq!(PlanningStage::ALL.len(), 9);
        assert_eq!(PlanningStage::Inputs.next(), Some(PlanningStage::Orient));
        assert_eq!(PlanningStage::Approve.next(), None);
    }
}
