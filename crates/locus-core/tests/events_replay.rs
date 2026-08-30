//! `agents.events` is the durable record a restart replays: persist, then read back
//! in capture order with payload fields intact.

use locus_core::ids::{EventId, RunId};
use locus_core::services::telemetry::{Event, EventVerb, Usage};
use locus_core::store::Store;
use sqlx::query;

#[tokio::test]
async fn events_replay_in_capture_order_with_payload_intact() {
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
        .expect("run migrations");

    // events.run_id references a real run; seed the project → agent → session → run
    // chain the same way schema_mail.rs does.
    let run_id = RunId::generate();
    query("INSERT INTO core.projects (id, name) VALUES ($1::uuid, 'events replay test')")
        .bind("00000000-0000-0000-0000-000000000201")
        .execute(store.test_pool())
        .await
        .expect("insert project");
    query(
        "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
         VALUES ($1::uuid, 'events replay agent', 1, '{}'::jsonb, 'test agent')",
    )
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await
    .expect("insert agent def");
    query(
        "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
         VALUES ($1::uuid, $2::uuid, $3::uuid, 'events replay session', 'agent/events-replay')",
    )
    .bind("00000000-0000-0000-0000-000000000203")
    .bind("00000000-0000-0000-0000-000000000201")
    .bind("00000000-0000-0000-0000-000000000202")
    .execute(store.test_pool())
    .await
    .expect("insert session");
    query(
        "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
         VALUES ($1::uuid, $2::uuid, 'test-model', 'running')",
    )
    .bind(run_id)
    .bind("00000000-0000-0000-0000-000000000203")
    .execute(store.test_pool())
    .await
    .expect("insert run");

    let first = Event {
        run_id,
        seq: 0,
        ts: "2026-08-30T01:00:00Z".into(),
        verb: EventVerb::User,
        text: Some("fix the flake".into()),
        tool: None,
        args: None,
        usage: None,
        raw: serde_json::json!({"method": "session/update"}),
    };
    let second = Event {
        run_id,
        seq: 1,
        ts: "2026-08-30T01:00:05Z".into(),
        verb: EventVerb::ToolResult,
        text: Some("1 file changed".into()),
        tool: Some("git".into()),
        args: Some(serde_json::json!({"command": "git push"})),
        usage: Some(Usage {
            input: Some(10),
            output: Some(4),
            cache_read: None,
            cache_write: None,
        }),
        raw: serde_json::json!({
            "method": "session/update",
            "params": {"update": {"sessionUpdate": "ToolCallUpdate", "status": "completed"}},
        }),
    };

    store
        .persist_events([(EventId::generate(), &first), (EventId::generate(), &second)])
        .await
        .expect("persist events");

    let replayed = store.events_for_run(run_id).await.expect("replay events");
    assert_eq!(replayed.len(), 2);
    assert_eq!([replayed[0].seq, replayed[1].seq], [0, 1]);
    assert_eq!(replayed[0].verb, EventVerb::User);
    assert_eq!(replayed[0].text.as_deref(), Some("fix the flake"));
    assert_eq!(replayed[1].verb, EventVerb::ToolResult);
    assert_eq!(replayed[1].tool.as_deref(), Some("git"));
    assert_eq!(
        replayed[1].args.as_ref(),
        Some(&serde_json::json!({"command": "git push"}))
    );
    assert_eq!(
        replayed[1].usage,
        Some(Usage {
            input: Some(10),
            output: Some(4),
            cache_read: None,
            cache_write: None,
        })
    );

    // Replay is scoped to its run: a fresh run id replays nothing.
    let other = store
        .events_for_run(RunId::generate())
        .await
        .expect("replay an empty run");
    assert!(other.is_empty());
}
