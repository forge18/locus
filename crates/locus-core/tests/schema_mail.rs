//! Schema coverage for `mail`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::{query, query_scalar};

use locus_core::store::Store;

#[tokio::test]
async fn schema_mail() {
    let (container, _cleanup) =
        locus_core::testkit::postgres::start_postgres_named("locus-postgres-test").await;
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");
    store
        .run_migrations(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
            &locus_core::testkit::postgres::NoopMigrationBackup,
            &locus_core::testkit::postgres::test_backup_config(),
        )
        .await
        .expect("run the mail migration");

    for table in ["threads", "messages", "deliveries", "waits"] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'mail' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.pool())
        .await
        .expect("query the mail schema");
        assert!(exists, "mail.{table} exists");
    }

    for (table, column) in [
        ("threads", "project_id"),
        ("threads", "subject"),
        ("messages", "thread_id"),
        ("messages", "sender_kind"),
        ("messages", "sender_run_id"),
        ("messages", "body"),
        ("deliveries", "message_id"),
        ("deliveries", "recipient_kind"),
        ("deliveries", "recipient_session_id"),
        ("deliveries", "status"),
        ("waits", "run_id"),
        ("waits", "reason"),
        ("waits", "started_at"),
        ("waits", "ended_at"),
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'mail' AND table_name = $1 AND column_name = $2
            )",
        )
        .bind(table)
        .bind(column)
        .fetch_one(store.pool())
        .await
        .expect("query mail schema columns");
        assert!(exists, "mail.{table}.{column} exists");
    }

    for index in [
        "mail_threads_project_id_idx",
        "mail_messages_thread_id_idx",
        "mail_deliveries_recipient_session_pending_idx",
        "mail_waits_active_run_key",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname = 'mail' AND indexname = $1
            )",
        )
        .bind(index)
        .fetch_one(store.pool())
        .await
        .expect("query mail indexes");
        assert!(exists, "mail index {index} exists");
    }

    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'mail test project')")
        .bind("00000000-0000-0000-0000-000000000101")
        .execute(store.pool())
        .await
        .expect("insert mail test project");
    query(
        "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
         VALUES ($1::uuid, 'mail test agent', 1, '{}'::jsonb, 'test agent')",
    )
    .bind("00000000-0000-0000-0000-000000000102")
    .execute(store.pool())
    .await
    .expect("insert mail test agent");
    query(
        "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
         VALUES ($1::uuid, $2::uuid, $3::uuid, 'mail test session', 'agent/mail-test')",
    )
    .bind("00000000-0000-0000-0000-000000000103")
    .bind("00000000-0000-0000-0000-000000000101")
    .bind("00000000-0000-0000-0000-000000000102")
    .execute(store.pool())
    .await
    .expect("insert mail test session");
    query(
        "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
         VALUES ($1::uuid, $2::uuid, 'test-model', 'running')",
    )
    .bind("00000000-0000-0000-0000-000000000104")
    .bind("00000000-0000-0000-0000-000000000103")
    .execute(store.pool())
    .await
    .expect("insert mail test run");

    query(
        "INSERT INTO mail.threads (id, project_id, subject)
         VALUES ($1::uuid, $2::uuid, 'mail test thread')",
    )
    .bind("00000000-0000-0000-0000-000000000105")
    .bind("00000000-0000-0000-0000-000000000101")
    .execute(store.pool())
    .await
    .expect("insert project-scoped mail thread");
    query(
        "INSERT INTO mail.messages (id, thread_id, sender_kind, sender_run_id, body)
         VALUES ($1::uuid, $2::uuid, 'agent', $3::uuid, 'mail test message')",
    )
    .bind("00000000-0000-0000-0000-000000000106")
    .bind("00000000-0000-0000-0000-000000000105")
    .bind("00000000-0000-0000-0000-000000000104")
    .execute(store.pool())
    .await
    .expect("insert threaded mail message");
    query(
        "INSERT INTO mail.deliveries (id, message_id, recipient_kind, recipient_session_id)
         VALUES ($1::uuid, $2::uuid, 'agent', $3::uuid)",
    )
    .bind("00000000-0000-0000-0000-000000000107")
    .bind("00000000-0000-0000-0000-000000000106")
    .bind("00000000-0000-0000-0000-000000000103")
    .execute(store.pool())
    .await
    .expect("insert agent delivery state");
    query(
        "INSERT INTO mail.waits (id, run_id, reason)
         VALUES ($1::uuid, $2::uuid, 'mail')",
    )
    .bind("00000000-0000-0000-0000-000000000108")
    .bind("00000000-0000-0000-0000-000000000104")
    .execute(store.pool())
    .await
    .expect("record mail wait reason");

    let second_active_wait = query(
        "INSERT INTO mail.waits (id, run_id, reason)
         VALUES ($1::uuid, $2::uuid, 'ask')",
    )
    .bind("00000000-0000-0000-0000-000000000109")
    .bind("00000000-0000-0000-0000-000000000104")
    .execute(store.pool())
    .await;
    assert!(
        second_active_wait.is_err(),
        "a run has at most one active wait"
    );

    let waiting_status = query("UPDATE agents.runs SET status = 'waiting' WHERE id = $1::uuid")
        .bind("00000000-0000-0000-0000-000000000104")
        .execute(store.pool())
        .await;
    assert!(
        waiting_status.is_err(),
        "waiting reasons are stored in mail.waits, not agents.runs.status"
    );

    query("DELETE FROM agents.runs WHERE id = $1::uuid")
        .bind("00000000-0000-0000-0000-000000000104")
        .execute(store.pool())
        .await
        .expect("delete a finished sender run");
    let sender_run_id: Option<String> =
        query_scalar("SELECT sender_run_id::text FROM mail.messages WHERE id = $1::uuid")
            .bind("00000000-0000-0000-0000-000000000106")
            .fetch_one(store.pool())
            .await
            .expect("read preserved mail message");
    assert_eq!(
        sender_run_id, None,
        "mail remains durable when its sender run is removed"
    );
}
