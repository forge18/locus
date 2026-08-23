//! Schema coverage for `wiki`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::query_scalar;

use locus_core::store::Store;

#[tokio::test]
async fn schema_wiki() {
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
        .expect("run the wiki migration");

    for table in [
        "pages",
        "revisions",
        "links",
        "contradictions",
        "ingest_log",
        "embeddings",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'wiki' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.pool())
        .await
        .expect("query the wiki schema");
        assert!(exists, "wiki.{table} exists");
    }

    let vector_available: bool =
        query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(store.pool())
            .await
            .expect("query the vector extension");
    assert!(vector_available, "pgvector is enabled for wiki embeddings");
}
