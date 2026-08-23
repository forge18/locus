//! Schema coverage for `pgvector_roundtrip`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::{query, query_scalar};

use locus_core::store::Store;

#[tokio::test]
async fn pgvector_roundtrip() {
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
        .expect("run migrations with the pgvector column");

    query(
        "INSERT INTO core.projects (id, name)
         VALUES ('00000000-0000-0000-0000-000000000001', 'pgvector roundtrip')",
    )
    .execute(store.test_pool())
    .await
    .expect("insert project for embeddings");
    query(
        "INSERT INTO memory.store (
             id, project_id, scope, path, subject, category, body, provenance,
             embedding, embedding_model, confidence, importance, strength
         ) VALUES
             (
                 '00000000-0000-0000-0000-000000000003',
                 '00000000-0000-0000-0000-000000000001',
                 'project',
                 'store.rs',
                 'expected nearest embedding',
                 'fact',
                 'The expected embedding is nearest to the query.',
                 '{}'::jsonb,
                 '[0.9,0.1,0.0]'::vector,
                 'test-model',
                 1.0,
                 1.0,
                 1.0
             ),
             (
                 '00000000-0000-0000-0000-000000000004',
                 '00000000-0000-0000-0000-000000000001',
                 'project',
                 'store.rs',
                 'far embedding',
                 'fact',
                 'This embedding is deliberately far from the query.',
                 '{}'::jsonb,
                 '[0.0,1.0,0.0]'::vector,
                 'test-model',
                 1.0,
                 1.0,
                 1.0
             )",
    )
    .execute(store.test_pool())
    .await
    .expect("insert pgvector embeddings");

    let nearest: String = query_scalar(
        "SELECT subject
         FROM memory.store
         ORDER BY embedding <-> '[0.9,0.1,0.0]'::vector
         LIMIT 1",
    )
    .fetch_one(store.test_pool())
    .await
    .expect("query nearest embedding");
    assert_eq!(nearest, "expected nearest embedding");
}
