mod dispatch {
    use locus_core::dispatch::DispatchPolicy;

    #[test]
    fn priority_policy() {
        let policy = DispatchPolicy::with_priority(locus_core::dispatch::PriorityMethod::Manual).expect("policy");
        assert_eq!(policy.priority_method, locus_core::dispatch::PriorityMethod::Manual);
        assert_eq!(policy.tie_break, locus_core::dispatch::TieBreak::LongestWaiting);
    }

    #[test]
    fn parallel_caps() {
        let policy = DispatchPolicy::with_parallelism(6, 3).expect("valid caps");
        assert_eq!((policy.global_parallelism, policy.per_project_parallelism), (6, 3));
        assert!(DispatchPolicy::with_parallelism(0, 3).is_err());
    }
}
