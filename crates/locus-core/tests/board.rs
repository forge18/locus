mod board {
    use locus_core::ids::{AgentDefId, EventId, ProjectId, RunId, SessionId, TaskId};
    use locus_core::services::board::{
        workflow_dependency_events, BoardActor, BoardError, BoardEvent, BoardEvidenceLink,
        BoardProjection, BoardTask,
    };
    use locus_core::services::manage::TaskColumn;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn task(column: TaskColumn) -> BoardTask {
        let mut task = BoardTask::new(
            ProjectId::generate(),
            TaskId::generate(),
            "task",
            Some("cargo test".into()),
        );
        task.column = column;
        task
    }

    fn proof() -> BoardEvidenceLink {
        BoardEvidenceLink {
            run_id: Some(RunId::generate()),
            event_ids: vec![EventId::generate()],
            artifact_ids: vec![],
            external: None,
        }
    }

    fn projection() -> (BoardProjection, BoardTask, BoardTask) {
        let predecessor = task(TaskColumn::Done);
        let dependent = task(TaskColumn::Testing);
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(predecessor.clone()),
            })
            .unwrap();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(dependent.clone()),
            })
            .unwrap();
        (projection, predecessor, dependent)
    }

    #[test]
    fn six_fixed_columns() {
        assert_eq!(TaskColumn::ALL.len(), 6);
        assert_eq!(TaskColumn::InProgress.as_str(), "in_progress");
    }

    #[test]
    fn columns_are_closed() {
        assert!(serde_json::from_str::<TaskColumn>("\"building\"").is_err());
        assert_eq!(
            serde_json::to_value(TaskColumn::Done).unwrap(),
            json!("done")
        );
    }

    #[test]
    fn task_shape() {
        let task = task(TaskColumn::Ready);
        assert!(task.verify_command.is_some());
        assert!(task.blocked_by.is_empty());
        assert!(task.evidence.is_empty());
        assert!(task.external_issue.is_none());
    }

    #[test]
    fn blocked_is_a_status() {
        let mut task = task(TaskColumn::Reviewing);
        task.block("reason", "predecessor completes");
        assert!(task.blocked);
        assert_eq!(task.column, TaskColumn::Reviewing);
    }

    #[test]
    fn blockable_anywhere() {
        for column in TaskColumn::ALL {
            let mut task = task(column);
            task.block("reason", "predecessor completes");
            assert!(task.blocked);
            assert_eq!(task.column, column);
        }
    }

    #[test]
    fn transitions() {
        let task = task(TaskColumn::Ready);
        let event = task
            .transition(TaskColumn::InProgress, BoardActor::Human, vec![])
            .unwrap();
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task.clone()),
            })
            .unwrap();
        projection.apply(event).unwrap();
        assert_eq!(
            projection.task(task.id).unwrap().column,
            TaskColumn::InProgress
        );
    }

    #[test]
    fn evidence_links() {
        assert!(proof().proves_done());
    }

    #[test]
    fn agent_done_needs_evidence() {
        let task = task(TaskColumn::Reviewing);
        assert!(matches!(
            task.transition(
                TaskColumn::Done,
                BoardActor::Agent {
                    run_id: RunId::generate()
                },
                vec![]
            ),
            Err(BoardError::AgentDoneNeedsEvidence { .. })
        ));
    }

    #[test]
    fn human_is_unrestricted() {
        let task = task(TaskColumn::Reviewing);
        let event = task
            .transition(TaskColumn::Done, BoardActor::Human, vec![])
            .unwrap();
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task),
            })
            .unwrap();
        projection.apply(event).unwrap();
    }

    #[test]
    fn edges_from_graph() {
        let (mut projection, predecessor, dependent) = projection();
        let graph: locus_core::services::workflow::graph::WorkflowGraph = serde_json::from_value(json!({
        "version": 1,
        "nodes": [
            {"id": "predecessor", "kind": "Task", "position": {"x": 0, "y": 0}},
            {"id": "dependent", "kind": "Task", "position": {"x": 10, "y": 0}}
        ],
        "edges": [{"id": "edge", "source": "predecessor", "sourceHandle": "out", "target": "dependent", "targetHandle": "in"}]
    })).unwrap();
        let ids = BTreeMap::from([
            ("predecessor".into(), predecessor.id),
            ("dependent".into(), dependent.id),
        ]);
        for event in workflow_dependency_events(&graph, &ids) {
            projection.apply(event).unwrap();
        }
        assert!(projection.task(dependent.id).unwrap().blocked);
    }

    #[test]
    fn no_manual_edges() {
        let event = BoardEvent::WorkflowDependency {
            task_id: TaskId::generate(),
            blocked_by: TaskId::generate(),
            workflow_node_id: "workflow-node".into(),
        };
        assert!(matches!(event, BoardEvent::WorkflowDependency { .. }));
    }

    #[test]
    fn auto_unblock() {
        let (mut projection, predecessor, dependent) = projection();
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "edge".into(),
            })
            .unwrap();
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .unwrap();
        assert!(!projection.task(dependent.id).unwrap().blocked);
    }

    #[test]
    fn unblock_does_not_move() {
        let (mut projection, predecessor, dependent) = projection();
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "edge".into(),
            })
            .unwrap();
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .unwrap();
        assert_eq!(
            projection.task(dependent.id).unwrap().column,
            TaskColumn::Testing
        );
    }

    #[test]
    fn no_manual_unblock() {
        assert_eq!(
            BoardProjection::default().clear_blocked_manually(TaskId::generate()),
            Err(BoardError::ManualBlockedClear)
        );
    }

    #[test]
    fn picked_up_automatically() {
        let (mut projection, predecessor, dependent) = projection();
        let agent = AgentDefId::generate();
        projection
            .apply(BoardEvent::Assigned {
                task_id: dependent.id,
                agent,
                actor: BoardActor::Human,
            })
            .unwrap();
        projection
            .apply(BoardEvent::WorkflowDependency {
                task_id: dependent.id,
                blocked_by: predecessor.id,
                workflow_node_id: "edge".into(),
            })
            .unwrap();
        projection
            .apply(BoardEvent::RunCompleted {
                task_id: predecessor.id,
                run_id: RunId::generate(),
                passed: true,
            })
            .unwrap();
        assert_eq!(projection.next_unblocked(agent), Some(dependent.id));
    }

    #[test]
    fn approval_is_an_inbox_item() {
        let mut task = task(TaskColumn::PendingApproval);
        task.session_id = Some(SessionId::generate());
        let mut projection = BoardProjection::default();
        projection
            .apply(BoardEvent::Created {
                task: Box::new(task),
            })
            .unwrap();
        assert_eq!(projection.pending_approvals().unwrap().len(), 1);
    }
}
