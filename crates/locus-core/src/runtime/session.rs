//! Durable session, ephemeral run, and prompt-response turn identities.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::services::telemetry::{Event, Usage};

/// A durable thread of work for one versioned agent definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub agent_def_id: Uuid,
    pub name: String,
    pub branch: String,
    pub board_task_id: Option<Uuid>,
    pub memory_base: Value,
    pub pane_state: Value,
    pub status: SessionStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Closed,
}

/// Session-owned context supplied whenever a fresh container starts after a reset.
#[derive(Clone, Debug, PartialEq)]
pub struct NextRun {
    pub run: Run,
    pub branch: String,
    pub board_task_id: Option<Uuid>,
    pub memory_base: Value,
}

/// Create a new container lifetime without discarding the durable session context.
pub fn start_next_run(session: &Session, resolved_model_id: impl Into<String>) -> NextRun {
    NextRun {
        run: Run {
            id: Uuid::new_v4(),
            session_id: session.id,
            resolved_model_id: resolved_model_id.into(),
            status: RunStatus::Queued,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        },
        branch: session.branch.clone(),
        board_task_id: session.board_task_id,
        memory_base: session.memory_base.clone(),
    }
}

/// The data used to prime a fresh harness process from Locus-owned history.
#[derive(Clone, Debug, PartialEq)]
pub struct ResumePlan {
    pub next_run: NextRun,
    pub prior_events: Vec<Event>,
}

/// Resume starts a new container and feeds it durable Locus events, never relying on a
/// harness-specific session implementation.
pub fn resume_from_events(
    session: &Session,
    events: impl IntoIterator<Item = Event>,
    resolved_model_id: impl Into<String>,
) -> ResumePlan {
    ResumePlan {
        next_run: start_next_run(session, resolved_model_id),
        prior_events: events.into_iter().collect(),
    }
}

/// One container lifetime within a session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub session_id: Uuid,
    pub resolved_model_id: String,
    pub status: RunStatus,
    /// Normalized records emitted during this container lifetime.
    pub events: Vec<Event>,
    /// Harness-reported token counts; absent means unknown, never zero.
    pub usage: Option<Usage>,
    /// Container process exit code once the run has ended.
    pub exit_code: Option<i32>,
    /// Why a cancelled run was stopped; absent for all other run states.
    pub cancel_reason: Option<String>,
    /// Harness-owned session identifier when the harness supplies one.
    pub native_session_id: Option<String>,
    /// Reviewable or reference deliverables produced by this run.
    pub artifacts: Vec<Artifact>,
}

/// A run-produced deliverable tracked independently from terminal output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub run_id: Uuid,
    pub kind: ArtifactKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Plan,
    Diff,
    Diagram,
    Image,
    Recording,
    Walkthrough,
    Finding,
    Payload,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Queued,
    Running,
    Paused,
    Stopped,
    Completed,
    Aborted,
    Cancelled,
}

/// One prompt and its eventual response within a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub id: Uuid,
    pub run_id: Uuid,
    pub ordinal: i64,
    pub prompt_event_id: Uuid,
    pub response_event_id: Option<Uuid>,
}

#[cfg(test)]
mod model {
    use super::{Artifact, ArtifactKind, Run, RunStatus, Session, SessionStatus, Turn};
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;
    use uuid::Uuid;

    use crate::store::{
        backup::{MigrationBackup, RetainedBackupConfig},
        {PostgresConfig, PostgresContainer, Store},
    };

    struct NoopMigrationBackup;

    impl MigrationBackup for NoopMigrationBackup {
        fn create_retained(&self, _: &RetainedBackupConfig) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["rm", "--force", &self.container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("docker")
                .args(["volume", "rm", "--force", &self.volume_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an unused local port");
        listener.local_addr().expect("read the local port").port()
    }

    #[test]
    fn session_run_turn_types_preserve_the_hierarchy() {
        let session_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let prompt_event_id = Uuid::new_v4();
        let response_event_id = Uuid::new_v4();
        let session = Session {
            id: session_id,
            project_id: Uuid::new_v4(),
            agent_def_id: Uuid::new_v4(),
            name: "model test".into(),
            branch: "agent/model-test".into(),
            board_task_id: None,
            memory_base: serde_json::json!({}),
            pane_state: serde_json::json!({}),
            status: SessionStatus::Active,
        };
        let run = Run {
            id: run_id,
            session_id: session.id,
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![Artifact {
                id: Uuid::new_v4(),
                run_id,
                kind: ArtifactKind::Plan,
            }],
        };
        let turn = Turn {
            id: Uuid::new_v4(),
            run_id: run.id,
            ordinal: 0,
            prompt_event_id,
            response_event_id: Some(response_event_id),
        };

        assert_eq!(run.session_id, session.id);
        assert_eq!(turn.run_id, run.id);
        assert_ne!(
            turn.prompt_event_id,
            turn.response_event_id.expect("response is present")
        );
    }

    #[tokio::test]
    async fn session_run_turn_tables_form_one_hierarchy() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-session-model-test-{suffix}");
        let volume_name = format!("locus-session-model-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container.start().await.expect("start PostgreSQL");
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
                &RetainedBackupConfig::new(
                    "postgres://locus@localhost/locus",
                    "/var/lib/locus/artifacts",
                    "/var/lib/locus/backups",
                ),
            )
            .await
            .expect("run migrations");

        for table in ["sessions", "runs", "turns"] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.tables
                    WHERE table_schema = 'agents' AND table_name = $1
                )",
            )
            .bind(table)
            .fetch_one(store.pool())
            .await
            .expect("query session model table");
            assert!(exists, "agents.{table} exists");
        }

        for (table, column, referenced_table) in [
            ("runs", "session_id", "sessions"),
            ("turns", "run_id", "runs"),
            ("turns", "prompt_event_id", "events"),
            ("turns", "response_event_id", "events"),
        ] {
            let exists: bool = query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.table_constraints AS constraints
                    JOIN information_schema.key_column_usage AS columns
                      ON constraints.constraint_name = columns.constraint_name
                     AND constraints.table_schema = columns.table_schema
                    JOIN information_schema.constraint_column_usage AS referenced
                      ON constraints.constraint_name = referenced.constraint_name
                     AND constraints.table_schema = referenced.table_schema
                    WHERE constraints.table_schema = 'agents'
                      AND constraints.table_name = $1
                      AND constraints.constraint_type = 'FOREIGN KEY'
                      AND columns.column_name = $2
                      AND referenced.table_name = $3
                )",
            )
            .bind(table)
            .bind(column)
            .bind(referenced_table)
            .fetch_one(store.pool())
            .await
            .expect("query session model foreign key");
            assert!(
                exists,
                "agents.{table}.{column} references agents.{referenced_table}"
            );
        }
    }
}

#[cfg(test)]
mod resume_without_native_id {
    use serde_json::json;
    use uuid::Uuid;

    use super::{resume_from_events, Session, SessionStatus};

    #[test]
    fn resume_needs_only_locus_owned_session_and_events() {
        let session = Session {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            agent_def_id: Uuid::new_v4(),
            name: "portable resume".into(),
            branch: "agent/portable-resume".into(),
            board_task_id: None,
            memory_base: json!({}),
            pane_state: json!({}),
            status: SessionStatus::Active,
        };

        let plan = resume_from_events(&session, [], "harness-without-native-session");

        assert_eq!(plan.next_run.run.session_id, session.id);
        assert!(plan.prior_events.is_empty());
    }
}

#[cfg(test)]
mod resume_from_events {
    use serde_json::json;
    use uuid::Uuid;

    use super::{resume_from_events, Session, SessionStatus};
    use crate::services::telemetry::{Event, EventVerb};

    #[test]
    fn primes_a_new_run_from_the_sessions_own_event_history() {
        let session = Session {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            agent_def_id: Uuid::new_v4(),
            name: "resume work".into(),
            branch: "agent/resume-work".into(),
            board_task_id: None,
            memory_base: json!({}),
            pane_state: json!({}),
            status: SessionStatus::Active,
        };
        let history = vec![Event {
            run_id: Uuid::new_v4().to_string(),
            seq: 0,
            ts: "2026-01-01T00:00:00Z".into(),
            verb: EventVerb::Assistant,
            text: Some("implemented the parser".into()),
            tool: None,
            args: None,
            usage: None,
            raw: json!({"source": "locus"}),
        }];

        let plan = resume_from_events(&session, history.clone(), "test-model");

        assert_ne!(plan.next_run.run.id, Uuid::nil());
        assert_eq!(plan.next_run.run.session_id, session.id);
        assert_eq!(plan.prior_events, history);
    }
}

#[cfg(test)]
mod survives_reset {
    use serde_json::json;
    use uuid::Uuid;

    use super::{start_next_run, Session, SessionStatus};

    #[test]
    fn second_run_inherits_the_session_context_after_a_reset() {
        let board_task_id = Uuid::new_v4();
        let session = Session {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            agent_def_id: Uuid::new_v4(),
            name: "resettable work".into(),
            branch: "agent/resettable-work".into(),
            board_task_id: Some(board_task_id),
            memory_base: json!({"focus": ["src/lib.rs"]}),
            pane_state: json!({}),
            status: SessionStatus::Active,
        };

        let next = start_next_run(&session, "test-model");

        assert_eq!(next.run.session_id, session.id);
        assert_eq!(next.branch, session.branch);
        assert_eq!(next.board_task_id, Some(board_task_id));
        assert_eq!(next.memory_base, session.memory_base);
    }
}

#[cfg(test)]
mod holds {
    use serde_json::json;
    use uuid::Uuid;

    use super::{Session, SessionStatus};

    #[test]
    fn session_retains_its_durable_context() {
        let agent_def_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let memory_base = json!({"paths": ["src/session.rs"]});
        let pane_state = json!({"kind": "terminal", "minimized": true});
        let session = Session {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            agent_def_id,
            name: "implementation".into(),
            branch: "agent/session-holds".into(),
            board_task_id: Some(task_id),
            memory_base: memory_base.clone(),
            pane_state: pane_state.clone(),
            status: SessionStatus::Active,
        };

        assert_eq!(session.agent_def_id, agent_def_id, "pins agent@version");
        assert_eq!(session.branch, "agent/session-holds");
        assert_eq!(session.board_task_id, Some(task_id));
        assert_eq!(session.memory_base, memory_base);
        assert_eq!(session.pane_state, pane_state);
    }
}

#[cfg(test)]
mod run {
    mod holds {
        use serde_json::json;
        use uuid::Uuid;

        use crate::{
            runtime::session::{Artifact, ArtifactKind, Run, RunStatus},
            services::telemetry::{Event, EventVerb, Usage},
        };

        #[test]
        fn run_retains_its_ephemeral_context() {
            let run_id = Uuid::new_v4();
            let events = vec![Event {
                run_id: run_id.to_string(),
                seq: 0,
                ts: "2026-01-01T00:00:00Z".into(),
                verb: EventVerb::SessionEnd,
                text: None,
                tool: None,
                args: None,
                usage: None,
                raw: json!({"type": "end"}),
            }];
            let usage = Usage {
                input: Some(100),
                output: Some(50),
                cache_read: Some(25),
                cache_write: Some(10),
            };
            let artifacts = vec![Artifact {
                id: Uuid::new_v4(),
                run_id,
                kind: ArtifactKind::Walkthrough,
            }];
            let run = Run {
                id: run_id,
                session_id: Uuid::new_v4(),
                resolved_model_id: "claude-opus-4-6".into(),
                status: RunStatus::Completed,
                events: events.clone(),
                usage: Some(usage.clone()),
                exit_code: Some(0),
                cancel_reason: None,
                native_session_id: None,
                artifacts: artifacts.clone(),
            };

            assert_eq!(run.events, events);
            assert_eq!(run.usage, Some(usage));
            assert_eq!(run.exit_code, Some(0));
            assert_eq!(run.artifacts, artifacts);
            assert_eq!(run.resolved_model_id, "claude-opus-4-6");
        }
    }
}
