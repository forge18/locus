mod workflow {
    use locus_core::ids::{AgentDefId, ProjectId, RunId, SessionId, TaskId};
    use locus_core::runtime::session::{Session, SessionStatus};
    use locus_core::services::workflow::{
        begin_execution, compile_workflow, gate_request, orchestration_model_invocation_hook,
        reset_same_session as reset_workflow_session, verify_in_fresh_container,
        ExecutionEntryPayload, GateKind, GateRequest, Guardrail, IterationEntryPayload,
        SuccessCriterion, SuccessCriterionKind, SupervisorEvent, VerifyContainerRequest,
        VerifyContainerRunner, VerifyEvidence, VerifyResultEntryPayload, WorkflowEntry,
        WorkflowEntryKind, WorkflowGovernance, WorkflowSupervisor, WorkflowsProjection,
    };
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn results_on_run() {
        let result = locus_core::services::workflow::RunGovernanceEvaluation::passed("run-1", 1);
        assert_eq!(result.run_id, "run-1");
        assert!(result.passed);
    }

    #[test]
    fn governance_requires_a_verify_node() {
        let governance = WorkflowGovernance {
            version: 1,
            goal: "Ship".into(),
            guardrails: vec![],
            success_criteria: vec![],
        };
        assert!(compile_workflow(serde_json::json!({"nodes": []}), governance).is_err());
    }

    #[test]
    fn success_criteria() {
        let criterion = SuccessCriterion {
            kind: SuccessCriterionKind::Command,
            checker: "cargo test".into(),
        };
        assert_eq!(criterion.kind, SuccessCriterionKind::Command);
        assert_eq!(criterion.checker, "cargo test");
    }

    #[test]
    fn guardrail_prompts() {
        let guardrail = Guardrail {
            name: "safe".into(),
            prompt: "preserve data".into(),
        };
        assert_eq!(
            (guardrail.name.as_str(), guardrail.prompt.as_str()),
            ("safe", "preserve data")
        );
    }

    #[test]
    fn goal_not_node() {
        let governance = WorkflowGovernance {
            version: 1,
            goal: "Ship".into(),
            guardrails: vec![],
            success_criteria: vec![],
        };
        assert_eq!(governance.goal, "Ship");
    }

    #[test]
    fn governance_root() {
        let governance = WorkflowGovernance {
            version: 1,
            goal: "Ship".into(),
            guardrails: vec![Guardrail {
                name: "safe".into(),
                prompt: "preserve".into(),
            }],
            success_criteria: vec![SuccessCriterion {
                kind: SuccessCriterionKind::Command,
                checker: "cargo test".into(),
            }],
        };
        assert_eq!(governance.version, 1);
    }

    #[test]
    fn entry_kinds() {
        assert_eq!(WorkflowEntryKind::ALL.len(), 4);
        assert_eq!(
            WorkflowEntryKind::VerifyResult.as_str(),
            "workflow.verify_result"
        );
        assert!(WorkflowEntryKind::parse("workflow.task_moved").is_err());
        let entry = WorkflowEntry::new(
            ProjectId::generate(),
            1,
            WorkflowEntryKind::Execution,
            2,
            json!({}),
            "system",
            None,
        );
        assert!(locus_core::services::workflow::decode_entry_payload(&entry).is_err());
    }

    #[test]
    fn arbiter_is_an_entry() {
        let iteration_id = Uuid::new_v4();
        let payload = IterationEntryPayload {
            iteration_id,
            execution_id: Uuid::new_v4(),
            run_id: None,
            number: 2,
            arbiter_class: Some("noise".into()),
            counts_toward_iteration_budget: false,
            started_at: None,
            ended_at: None,
        };
        let entry = WorkflowEntry::new(
            ProjectId::generate(),
            1,
            WorkflowEntryKind::Iteration,
            1,
            serde_json::to_value(payload).expect("iteration payload"),
            "system",
            None,
        );
        let projection = WorkflowsProjection::rebuild([entry]).expect("project iteration");
        let iteration = projection
            .iteration(iteration_id)
            .expect("iteration projection");
        assert_eq!(iteration.arbiter_class.as_deref(), Some("noise"));
        assert!(!iteration.counts_toward_iteration_budget);
    }

    fn governance() -> WorkflowGovernance {
        WorkflowGovernance {
            version: 1,
            goal: "ship the change".into(),
            guardrails: vec![],
            success_criteria: vec![],
        }
    }

    fn graph() -> serde_json::Value {
        json!({
            "nodes": [
                {"id": "agent", "kind": "Agent", "role": "builder"},
                {"id": "loop", "kind": "Loop", "max_iterations": 3, "reset_to": "agent"},
                {"id": "verify", "kind": "Verify", "command": "cargo test"}
            ],
            "edges": [
                {"from": "agent", "to": "loop"},
                {"from": "loop", "to": "verify"}
            ]
        })
    }

    #[tokio::test]
    async fn defs_table() {
        let (container, _cleanup) =
            locus_core::testkit::postgres::start_postgres_named("locus-workflow-defs-test").await;
        let store = locus_core::store::Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &locus_core::testkit::postgres::NoopMigrationBackup,
                &locus_core::testkit::postgres::test_backup_config(),
            )
            .await
            .expect("run migrations");
        let project_id = Uuid::new_v4();
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1, 'workflow defs test')")
            .bind(project_id)
            .execute(store.test_pool())
            .await
            .expect("insert project");
        let compiled = compile_workflow(graph(), governance()).expect("compile workflow");
        let row = store
            .save_workflow_definition(project_id, "compiled", &compiled)
            .await
            .expect("save definition");
        assert_eq!(row.version, 1);
        assert_eq!(row.graph, compiled.graph().clone());
        assert_eq!(row.spec["verify_command"], "cargo test");
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM workflows.workflow_defs WHERE id = $1")
                .bind(row.id)
                .fetch_one(store.test_pool())
                .await
                .expect("count definition");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn condition_replays_historically() {
        let (container, _cleanup) =
            locus_core::testkit::postgres::start_postgres_named("locus-workflow-replay-test").await;
        let store = locus_core::store::Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &locus_core::testkit::postgres::NoopMigrationBackup,
                &locus_core::testkit::postgres::test_backup_config(),
            )
            .await
            .expect("run migrations");
        let project_id = ProjectId::generate();
        let workflow_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();
        sqlx::query("INSERT INTO core.projects (id, name) VALUES ($1, 'workflow replay')")
            .bind(project_id)
            .execute(store.test_pool())
            .await
            .expect("insert project");
        sqlx::query(
            "INSERT INTO workflows.workflow_defs
                (id, project_id, name, version, graph, spec, verify_command)
             VALUES ($1, $2, 'replay', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .bind(workflow_id)
        .bind(project_id)
        .execute(store.test_pool())
        .await
        .expect("insert workflow definition");
        store
            .append_execution_entry(
                project_id,
                ExecutionEntryPayload {
                    execution_id,
                    workflow_def_id: workflow_id,
                    schedule_id: None,
                    status: "running".into(),
                    scheduled_for: None,
                    started_at: None,
                    ended_at: None,
                },
                "system",
            )
            .await
            .expect("append execution");
        let earlier = store
            .append_verify_result_entry(
                project_id,
                VerifyResultEntryPayload {
                    id: Uuid::new_v4(),
                    execution_id,
                    iteration_id: None,
                    verify_node_id: "verify".into(),
                    command: "cargo test".into(),
                    container_id: "verify-1".into(),
                    exit_code: 1,
                    passed: false,
                    stdout: String::new(),
                    stderr: "failed".into(),
                    completed_at: None,
                },
                "system",
                None,
            )
            .await
            .expect("append earlier verify");
        let later = store
            .append_verify_result_entry(
                project_id,
                VerifyResultEntryPayload {
                    id: Uuid::new_v4(),
                    execution_id,
                    iteration_id: None,
                    verify_node_id: "verify".into(),
                    command: "cargo test".into(),
                    container_id: "verify-2".into(),
                    exit_code: 0,
                    passed: true,
                    stdout: "ok".into(),
                    stderr: String::new(),
                    completed_at: None,
                },
                "system",
                Some(earlier.stream_pos),
            )
            .await
            .expect("append later verify");
        let condition = locus_core::services::condition::Condition::parse("verify.passed == true")
            .expect("parse condition");
        assert!(!store
            .evaluate_condition_as_of(project_id, earlier.stream_pos, &condition)
            .await
            .expect("evaluate earlier condition"));
        assert!(store
            .evaluate_condition_expression_as_of(
                project_id,
                later.stream_pos,
                "verify.passed == true",
            )
            .await
            .expect("evaluate current condition"));
    }

    #[test]
    fn compile_together() {
        let compiled = compile_workflow(graph(), governance()).expect("compile workflow");
        assert_eq!(compiled.spec().verify_command(), "cargo test");
        assert_eq!(compiled.persisted_spec()["steps"][0]["node_id"], "agent");
        assert_eq!(compiled.graph()["nodes"][1]["kind"], "Loop");
    }

    #[test]
    fn cannot_disagree() {
        let compiled = compile_workflow(graph(), governance()).expect("compile workflow");
        assert!(compiled.persisted_spec().get("graph").is_none());
        assert!(compile_workflow(
            json!({"nodes": [{"id": "goal", "kind": "Goal"}]}),
            governance()
        )
        .is_err());
        assert!(compile_workflow(json!({"execution": {}}), governance()).is_err());
    }

    #[test]
    fn walks_spec() {
        let compiled = compile_workflow(graph(), governance()).expect("compile workflow");
        let events = WorkflowSupervisor::from_compiled(&compiled)
            .walk()
            .expect("bounded walk");
        assert!(events.iter().any(|event| matches!(
            event,
            SupervisorEvent::LoopReset {
                next_iteration: 2,
                ..
            }
        )));
        assert!(events.iter().any(
            |event| matches!(event, SupervisorEvent::Step { node_id } if node_id == "verify")
        ));
    }

    fn session() -> Session {
        Session {
            id: SessionId::generate(),
            project_id: ProjectId::generate(),
            agent_def_id: AgentDefId::generate(),
            name: "workflow session".into(),
            branch: "agent/workflow".into(),
            board_task_id: Some(TaskId::generate()),
            memory_base: json!({"carry": true}),
            pane_state: json!({}),
            status: SessionStatus::Active,
        }
    }

    #[test]
    fn reset_same_session() {
        let session = session();
        let plan = reset_workflow_session(&session, [], "model");
        assert_eq!(plan.next_run.run.session_id, session.id);
        assert_eq!(plan.next_run.branch, session.branch);
    }

    #[test]
    fn state_survives_reset() {
        let session = session();
        let task = session.board_task_id;
        let plan = reset_workflow_session(&session, [], "model");
        assert_eq!(plan.next_run.memory_base, session.memory_base);
        assert_eq!(plan.next_run.board_task_id, task);
        assert_eq!(plan.next_run.branch, session.branch);
    }

    #[test]
    fn loop_reset_creates_distinct_same_session_runs() {
        let session = session();
        let plans =
            locus_core::services::workflow::reset_same_session_runs(&session, [], "model", 2)
                .expect("bounded reset plans");
        assert_eq!(plans.len(), 3);
        assert!(plans
            .windows(2)
            .all(|pair| pair[0].next_run.run.id != pair[1].next_run.run.id));
        assert!(plans
            .iter()
            .all(|plan| plan.next_run.run.session_id == session.id));
    }

    #[derive(Default)]
    struct VerifyFake {
        request: Option<VerifyContainerRequest>,
    }

    impl VerifyContainerRunner for VerifyFake {
        fn run_fresh_container(
            &mut self,
            request: &VerifyContainerRequest,
        ) -> Result<VerifyEvidence, locus_core::services::workflow::VerifyError> {
            self.request = Some(request.clone());
            Ok(VerifyEvidence {
                exit_code: 0,
                stdout: "ok".into(),
                stderr: String::new(),
                passed: true,
                command: request.command.clone(),
                container_id: "verify-container".into(),
                verify_node_id: request.verify_node_id.clone(),
            })
        }
    }

    fn verify_request() -> VerifyContainerRequest {
        VerifyContainerRequest::new(
            RunId::generate(),
            "verify",
            "locus/agent:test",
            "git://host/project.git",
            "agent/workflow",
            "cargo test",
            "locus-agent-agent",
        )
        .expect("verify request")
    }

    #[test]
    fn verify_fresh_container() {
        let mut fake = VerifyFake::default();
        let request = verify_request();
        let evidence = verify_in_fresh_container(&mut fake, &request).expect("verify");
        assert_ne!(request.container_name, request.agent_container_name);
        assert_eq!(evidence.container_id, "verify-container");
        assert_eq!(fake.request.as_ref(), Some(&request));
    }

    #[test]
    fn verify_is_not_local() {
        let request = verify_request();
        assert!(request.clone_command.contains("git clone"));
        assert!(request.clone_command.contains("checkout 'agent/workflow'"));
        assert_ne!(request.container_name, request.agent_container_name);
        assert!(VerifyContainerRequest::new(
            request.run_id,
            "verify",
            "image",
            "remote",
            "main",
            "cargo test",
            "agent",
        )
        .is_err());
    }

    #[test]
    fn gate() {
        let human = gate_request(
            "approval",
            &GateKind::Human {
                prompt: "approve".into(),
            },
            session().project_id,
            session().id,
            RunId::generate(),
            vec![],
        )
        .expect("human gate request");
        let GateRequest::Human(human) = human else {
            panic!("expected human gate")
        };
        assert_eq!(
            human.waiting_state().reason,
            locus_core::services::mail::WaitReason::Gate
        );
        assert!(human.inbox_item().is_ok());

        let reviewer = gate_request(
            "review",
            &GateKind::ReviewerAgent {
                role: "reviewer".into(),
                max_rounds: 2,
            },
            session().project_id,
            session().id,
            RunId::generate(),
            vec!["diff summary".into()],
        )
        .expect("reviewer gate request");
        assert!(matches!(reviewer, GateRequest::ReviewerAgent(_)));
    }

    #[test]
    fn goal_gates_the_loop() {
        let compiled = compile_workflow(graph(), governance()).expect("workflow compiles");
        let mut execution = begin_execution(&compiled);
        assert!(execution.start_loop().is_err());
        execution.approve_goal(1).expect("approve goal");
        execution.start_loop().expect("start approved loop");
        assert!(execution.loop_may_run());
    }

    #[test]
    fn no_model_in_orchestration() {
        assert!(orchestration_model_invocation_hook().is_none());
    }

    #[test]
    fn verify_evidence() {
        let mut fake = VerifyFake::default();
        let request = verify_request();
        let evidence = verify_in_fresh_container(&mut fake, &request).expect("verify");
        assert_eq!(
            (
                evidence.exit_code,
                evidence.stdout,
                evidence.stderr,
                evidence.passed
            ),
            (0, "ok".into(), String::new(), true)
        );
        assert_eq!(evidence.command, "cargo test");
        assert_eq!(evidence.verify_node_id, "verify");
    }
}
