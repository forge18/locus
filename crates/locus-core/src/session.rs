//! Durable session, ephemeral run, and prompt-response turn identities.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A durable thread of work for one versioned agent definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub project_id: Uuid,
    pub agent_def_id: Uuid,
    pub name: String,
    pub branch: String,
    pub status: SessionStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Closed,
}

/// One container lifetime within a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub session_id: Uuid,
    pub resolved_model_id: String,
    pub status: RunStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Queued,
    Running,
    Paused,
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
    use super::{Run, RunStatus, Session, SessionStatus, Turn};
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::query_scalar;
    use uuid::Uuid;

    use crate::{
        backup::{MigrationBackup, RetainedBackupConfig},
        store::{PostgresConfig, PostgresContainer, Store},
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
            status: SessionStatus::Active,
        };
        let run = Run {
            id: run_id,
            session_id: session.id,
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
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
