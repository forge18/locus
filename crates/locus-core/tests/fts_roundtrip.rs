//! Schema coverage for `fts_roundtrip`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::{query, query_scalar};

use locus_core::store::Store;

#[tokio::test]
async fn fts_roundtrip() {
    let (container, _cleanup) =
        locus_core::testkit::postgres::start_postgres_named("locus-postgres-test").await;
    let store = Store::connect(&container.database_url())
        .await
        .expect("connect the store pool");

    query(
        "CREATE TABLE fts_documents (
            id INTEGER PRIMARY KEY,
            body TEXT NOT NULL,
            search tsvector GENERATED ALWAYS AS (to_tsvector('english', body)) STORED
        )",
    )
    .execute(store.pool())
    .await
    .expect("create a document table with a tsvector column");
    query("CREATE INDEX fts_documents_search_idx ON fts_documents USING GIN (search)")
        .execute(store.pool())
        .await
        .expect("create the tsvector index");
    query(
        "INSERT INTO fts_documents (id, body) VALUES
            (1, 'The PostgreSQL full text index returns this matching document.'),
            (2, 'A different document does not contain the search term.')",
    )
    .execute(store.pool())
    .await
    .expect("insert full-text documents");

    let matching_id: i32 = query_scalar(
        "SELECT id
         FROM fts_documents
         WHERE search @@ websearch_to_tsquery('english', 'PostgreSQL full text index')",
    )
    .fetch_one(store.pool())
    .await
    .expect("query the matching full-text document");
    assert_eq!(matching_id, 1);
}
