//! Schema coverage for `workflows`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::{query, query_scalar};

use locus_core::store::Store;

#[tokio::test]
async fn schema_workflows() {
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
        .expect("run the workflows migration");

    for table in [
        "workflow_defs",
        "schedules",
        "executions",
        "iterations",
        "guardrail_trips",
        "verify_results",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'workflows' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.test_pool())
        .await
        .expect("query the workflows schema");
        assert!(exists, "workflows.{table} exists");
    }

    for column in [
        "project_id",
        "name",
        "version",
        "graph",
        "spec",
        "verify_command",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'workflows'
                    AND table_name = 'workflow_defs'
                    AND column_name = $1
            )",
        )
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query workflow definition columns");
        assert!(exists, "workflows.workflow_defs.{column} exists");
    }

    for (table, column) in [
        ("schedules", "cron_expression"),
        ("schedules", "paused_at"),
        ("executions", "status"),
        ("executions", "schedule_id"),
        ("iterations", "arbiter_class"),
        ("iterations", "counts_toward_iteration_budget"),
        ("guardrail_trips", "guardrail"),
        ("verify_results", "passed"),
        ("verify_results", "exit_code"),
        ("verify_results", "stdout"),
        ("verify_results", "stderr"),
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'workflows' AND table_name = $1 AND column_name = $2
            )",
        )
        .bind(table)
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query workflow lifecycle columns");
        assert!(exists, "workflows.{table}.{column} exists");
    }

    for index in [
        "workflow_defs_project_name_version_key",
        "schedules_workflow_def_id_idx",
        "executions_schedule_id_idx",
        "executions_active_schedule_idx",
        "iterations_execution_number_key",
        "guardrail_trips_execution_id_idx",
        "verify_results_execution_id_idx",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname = 'workflows' AND indexname = $1
            )",
        )
        .bind(index)
        .fetch_one(store.test_pool())
        .await
        .expect("query workflow indexes");
        assert!(exists, "workflow index {index} exists");
    }

    for table in ["project_streams", "entries"] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'log' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.test_pool())
        .await
        .expect("query the workflow log schema");
        assert!(exists, "log.{table} exists");
    }
    for column in [
        "project_id",
        "stream_pos",
        "kind",
        "v",
        "payload",
        "actor",
        "caused_by",
        "created_at",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = 'log' AND table_name = 'entries' AND column_name = $1
            )",
        )
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query workflow log columns");
        assert!(exists, "log.entries.{column} exists");
    }

    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'workflow test project')")
        .bind("00000000-0000-0000-0000-000000000001")
        .execute(store.test_pool())
        .await
        .expect("insert workflow test project");
    query(
        "INSERT INTO workflows.workflow_defs
            (id, project_id, name, version, graph, spec, verify_command)
         VALUES
            ($1::uuid, $2::uuid, 'test workflow', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
    )
    .bind("00000000-0000-0000-0000-000000000002")
    .bind("00000000-0000-0000-0000-000000000001")
    .execute(store.test_pool())
    .await
    .expect("insert immutable workflow definition");
    let update =
        query("UPDATE workflows.workflow_defs SET name = 'changed workflow' WHERE id = $1::uuid")
            .bind("00000000-0000-0000-0000-000000000002")
            .execute(store.test_pool())
            .await;
    assert!(update.is_err(), "workflow definitions are immutable");
}
