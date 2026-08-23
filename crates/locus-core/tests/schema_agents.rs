//! Schema coverage for `agents`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::query_scalar;

use locus_core::store::Store;

#[tokio::test]
async fn schema_agents() {
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
        .expect("run the agents migration");

    for table in [
        "agent_defs",
        "sessions",
        "runs",
        "run_edges",
        "events",
        "artifacts",
        "artifact_comments",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'agents' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.test_pool())
        .await
        .expect("query the agents schema");
        assert!(exists, "agents.{table} exists");
    }
}
