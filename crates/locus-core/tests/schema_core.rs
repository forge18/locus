//! Schema coverage for `core`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::query_scalar;

use locus_core::store::Store;

#[tokio::test]
async fn schema_core() {
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
        .expect("run the core migration");

    for table in ["projects", "repos", "local_remotes", "settings"] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'core' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.pool())
        .await
        .expect("query the core schema");
        assert!(exists, "core.{table} exists");
    }
}
