//! Schema coverage for `memory`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::query_scalar;

use locus_core::store::Store;

#[tokio::test]
async fn schema_memory() {
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
        .expect("run the memory migration");

    for table in ["core", "store", "probation", "edges"] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'memory' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.test_pool())
        .await
        .expect("query the memory schema");
        assert!(exists, "memory.{table} exists");
    }

    for column in [
        "project_id",
        "agent_def_id",
        "scope",
        "provenance",
        "embedding",
        "confidence",
        "importance",
        "recall_count",
        "active_days",
        "strength",
        "eviction_class",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'memory' AND table_name = 'store' AND column_name = $1
            )",
        )
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query durable memory columns");
        assert!(exists, "memory.store.{column} exists");
    }

    let embedding_type: String = query_scalar(
        "SELECT udt_name
            FROM information_schema.columns
            WHERE table_schema = 'memory' AND table_name = 'store' AND column_name = 'embedding'",
    )
    .fetch_one(store.test_pool())
    .await
    .expect("query the durable memory embedding type");
    assert_eq!(
        embedding_type, "vector",
        "memory.store.embedding uses pgvector"
    );

    let attribution_view: bool = query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.views
            WHERE table_schema = 'agents' AND table_name = 'context_attribution'
        )",
    )
    .fetch_one(store.test_pool())
    .await
    .expect("query context attribution view");
    assert!(attribution_view, "agents.context_attribution exists");

    for column in [
        "context_event",
        "materialization_snapshot",
        "verification_duration_ms",
        "verification_tokens",
        "tool_result_event_id",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'agents' AND table_name = 'context_attribution' AND column_name = $1
            )",
        )
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query context attribution columns");
        assert!(exists, "agents.context_attribution.{column} exists");
    }

    for index in [
        "memory_core_project_agent_idx",
        "memory_store_project_scope_idx",
        "memory_store_project_path_idx",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname = 'memory' AND indexname = $1
            )",
        )
        .bind(index)
        .fetch_one(store.test_pool())
        .await
        .expect("query memory indexes");
        assert!(exists, "memory index {index} exists");
    }
}
