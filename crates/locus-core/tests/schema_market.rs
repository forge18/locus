//! Schema coverage for `market`. Moved out of `store/mod.rs`:
//! these drive a real Postgres container and assert on tables, not on private items.

use sqlx::{query, query_scalar};

use locus_core::store::Store;

#[tokio::test]
async fn schema_market() {
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
        .expect("run the market migration");

    for table in [
        "manifest_snapshots",
        "installs",
        "tool_sets",
        "tool_set_manifest_pins",
        "agent_tool_set_resolutions",
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'market' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(store.test_pool())
        .await
        .expect("query the market schema");
        assert!(exists, "market.{table} exists");
    }

    for (table, column) in [
        ("manifest_snapshots", "name"),
        ("manifest_snapshots", "manifest"),
        ("manifest_snapshots", "content_sha256"),
        ("installs", "tool_set_id"),
        ("installs", "manifest_snapshot_id"),
        ("installs", "status"),
        ("tool_sets", "base_image_digest"),
        ("tool_sets", "image_cache_key"),
        ("tool_set_manifest_pins", "tool_name"),
        ("tool_set_manifest_pins", "manifest_snapshot_id"),
        ("agent_tool_set_resolutions", "agent_def_id"),
        ("agent_tool_set_resolutions", "tool_set_id"),
    ] {
        let exists: bool = query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'market' AND table_name = $1 AND column_name = $2
            )",
        )
        .bind(table)
        .bind(column)
        .fetch_one(store.test_pool())
        .await
        .expect("query market schema columns");
        assert!(exists, "market.{table}.{column} exists");
    }

    query(
        "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
         VALUES ($1::uuid, 'market test agent', 1, '{}'::jsonb, 'test agent')",
    )
    .bind("00000000-0000-0000-0000-000000000201")
    .execute(store.test_pool())
    .await
    .expect("insert market test agent");
    query(
        "INSERT INTO market.manifest_snapshots (id, name, manifest, content_sha256)
         VALUES ($1::uuid, 'amq',
                 '{\"name\": \"amq\", \"install\": {\"cargo\": \"agent-message-queue\"}}'::jsonb,
                 'a3f7b1')",
    )
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await
    .expect("cache a manifest snapshot");
    query(
        "INSERT INTO market.tool_sets (id, base_image_digest, image_cache_key)
         VALUES ($1::uuid, 'sha256:base-image', 'sha256:agent-image')",
    )
    .bind("00000000-0000-0000-0000-000000000203")
    .execute(store.test_pool())
    .await
    .expect("create deterministic image tool set");
    query(
        "INSERT INTO market.tool_set_manifest_pins (tool_set_id, tool_name, manifest_snapshot_id)
         VALUES ($1::uuid, 'amq', $2::uuid)",
    )
    .bind("00000000-0000-0000-0000-000000000203")
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await
    .expect("pin manifest snapshot in tool set");
    query(
        "INSERT INTO market.installs (id, tool_set_id, manifest_snapshot_id, status)
         VALUES ($1::uuid, $2::uuid, $3::uuid, 'verified')",
    )
    .bind("00000000-0000-0000-0000-000000000204")
    .bind("00000000-0000-0000-0000-000000000203")
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await
    .expect("record an install for a pinned manifest");
    query(
        "INSERT INTO market.agent_tool_set_resolutions (agent_def_id, tool_set_id)
         VALUES ($1::uuid, $2::uuid)",
    )
    .bind("00000000-0000-0000-0000-000000000201")
    .bind("00000000-0000-0000-0000-000000000203")
    .execute(store.test_pool())
    .await
    .expect("resolve the agent definition to its tool set");

    let duplicate_cache_key = query(
        "INSERT INTO market.tool_sets (id, base_image_digest, image_cache_key)
         VALUES ($1::uuid, 'sha256:base-image', 'sha256:agent-image')",
    )
    .bind("00000000-0000-0000-0000-000000000205")
    .execute(store.test_pool())
    .await;
    assert!(
        duplicate_cache_key.is_err(),
        "one deterministic cache key identifies one tool set"
    );

    let duplicate_resolution = query(
        "INSERT INTO market.agent_tool_set_resolutions (agent_def_id, tool_set_id)
         VALUES ($1::uuid, $2::uuid)",
    )
    .bind("00000000-0000-0000-0000-000000000201")
    .bind("00000000-0000-0000-0000-000000000203")
    .execute(store.test_pool())
    .await;
    assert!(
        duplicate_resolution.is_err(),
        "an immutable agent definition has one resolved tool set"
    );

    let mismatched_pin = query(
        "INSERT INTO market.tool_set_manifest_pins (tool_set_id, tool_name, manifest_snapshot_id)
         VALUES ($1::uuid, 'not-amq', $2::uuid)",
    )
    .bind("00000000-0000-0000-0000-000000000203")
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await;
    assert!(
        mismatched_pin.is_err(),
        "a tool-set pin must use the snapshot's tool name"
    );
}
