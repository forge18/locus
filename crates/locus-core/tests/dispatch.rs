mod dispatch {
    use locus_core::ids::RunId;
    use locus_core::ids::SessionId;
    use locus_core::runtime::dispatch::DispatchPolicy;

    #[test]
    fn restore_window() {
        assert_eq!(
            locus_core::runtime::dispatch::StopAllSnapshot::restore_window_minutes(),
            10
        );
    }

    #[test]
    fn stop_all_preserves_work() {
        let snapshot = locus_core::runtime::dispatch::StopAllSnapshot {
            id: uuid::Uuid::nil(),
            run_ids: vec![RunId::generate()],
        };
        assert!(snapshot.preserves_durable_work());
    }

    #[test]
    fn stop_all_snapshot() {
        let snapshot = locus_core::runtime::dispatch::StopAllSnapshot {
            id: uuid::Uuid::nil(),
            run_ids: vec![],
        };
        assert!(snapshot.is_empty());
    }

    #[test]
    fn autorun_state() {
        let state = locus_core::runtime::dispatch::AutorunState::enabled();
        assert!(state.is_enabled());
    }

    #[test]
    fn preemption_handoff() {
        let handoff = locus_core::runtime::dispatch::PreemptionHandoff {
            session_id: SessionId::default(),
            branch: "agent/task".into(),
            board_task_id: None,
            memory_base: serde_json::json!({"decision": "keep"}),
        };
        assert!(handoff.retains_context());
    }

    #[test]
    fn preempts_at_boundary() {
        let controller = locus_core::runtime::dispatch::PreemptionController::default();
        assert!(!controller.has_pending_preemption());
    }

    #[test]
    fn queues_at_cap() {
        let policy = DispatchPolicy::with_parallelism(1, 1).expect("caps");
        assert!(locus_core::runtime::dispatch::queues_at_cap(&policy, 1));
        assert!(!locus_core::runtime::dispatch::queues_at_cap(&policy, 0));
    }

    #[test]
    fn priority_policy() {
        let policy =
            DispatchPolicy::with_priority(locus_core::runtime::dispatch::PriorityMethod::Manual)
                .expect("policy");
        assert_eq!(
            policy.priority_method,
            locus_core::runtime::dispatch::PriorityMethod::Manual
        );
        assert_eq!(
            policy.tie_break,
            locus_core::runtime::dispatch::TieBreak::LongestWaiting
        );
    }

    #[test]
    fn parallel_caps() {
        let policy = DispatchPolicy::with_parallelism(6, 3).expect("valid caps");
        assert_eq!(
            (policy.global_parallelism, policy.per_project_parallelism),
            (6, 3)
        );
        assert!(DispatchPolicy::with_parallelism(0, 3).is_err());
    }
}
