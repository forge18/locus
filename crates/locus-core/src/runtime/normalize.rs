//! Capture records in, ordered durable events out.
//!
//! Kept in `runtime` rather than `services::telemetry` because persisting is part of it,
//! and a shared service must not know about the store. The event vocabulary and the ACP
//! adapter live in `services::telemetry`; wiring them to a run lives here.

use crate::ids::{EventId, RunId};
use anyhow::Result;
use serde_json::Value;

use crate::{
    services::telemetry::{Adapter, CapturedEvent, Event, EventCollector},
    store::Store,
};

/// Normalize captured source records through the adapter selected for this run's telemetry source.
pub fn normalize(
    adapter: &dyn Adapter,
    records: impl IntoIterator<Item = Value>,
) -> Result<Vec<CapturedEvent>> {
    records
        .into_iter()
        .try_fold(Vec::new(), |mut events, record| {
            events.extend(adapter.normalize(record)?);
            Ok(events)
        })
}

/// Assign run-owned ordering and durably store every normalized event before returning it.
pub async fn persist_normalized_events(
    store: &Store,
    collector: &EventCollector,
    run_id: RunId,
    captured: impl IntoIterator<Item = CapturedEvent>,
) -> Result<Vec<Event>> {
    let events = captured
        .into_iter()
        .map(|event| collector.capture(run_id, event))
        .collect::<Vec<_>>();

    store
        .persist_events(events.iter().map(|event| (EventId::generate(), event)))
        .await?;

    Ok(events)
}

/// Normalize two live sources through the same collector without exposing their harness dialects
/// after the capture boundary.
pub async fn normalize_two_harnesses(
    collector: &EventCollector,
    first_run_id: RunId,
    first_adapter: &dyn Adapter,
    first_records: Vec<Value>,
    second_run_id: RunId,
    second_adapter: &dyn Adapter,
    second_records: Vec<Value>,
) -> Result<Vec<Event>> {
    let (first, second) = tokio::join!(async { normalize(first_adapter, first_records) }, async {
        normalize(second_adapter, second_records)
    },);
    let first = first?;
    let second = second?;
    Ok(first
        .into_iter()
        .map(|event| collector.capture(first_run_id, event))
        .chain(
            second
                .into_iter()
                .map(|event| collector.capture(second_run_id, event)),
        )
        .collect())
}

#[cfg(test)]
mod two_harnesses_concurrent {
    use crate::ids::RunId;
    use serde_json::json;

    use super::normalize_two_harnesses;
    use crate::services::telemetry::{AcpAdapter, EventCollector, EventVerb};

    #[tokio::test]
    async fn concurrent_harnesses_emit_the_same_downstream_event_shape() {
        // One mapping for every ACP harness: two concurrent runs differ by run id and
        // sequence, never by dialect.
        let first = AcpAdapter;
        let second = AcpAdapter;
        let collector = EventCollector::new(4);

        let events = normalize_two_harnesses(
            &collector,
            RunId::generate(),
            &first,
            vec![json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"one"}}}})],
            RunId::generate(),
            &second,
            vec![json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"two"}}}})],
        )
        .await
        .expect("both sources normalize");

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].verb, EventVerb::Assistant);
        assert_eq!(events[1].verb, EventVerb::Assistant);
        assert_eq!(events[0].tool, events[1].tool);
        assert_eq!(events[0].args, events[1].args);
    }
}

#[cfg(test)]
mod normalizes {
    use serde_json::json;

    use super::normalize;
    use crate::services::telemetry::{AcpAdapter, EventVerb};

    #[test]
    fn hands_captured_records_to_the_source_adapter() {
        let adapter = AcpAdapter;
        let raw = json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"complete"}}}});

        let events = normalize(&adapter, [raw.clone()]).expect("captured record normalizes");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].verb, EventVerb::Assistant);
        assert_eq!(events[0].raw, raw);
    }
}

#[cfg(test)]
mod persists_events {
    use crate::ids::EventId;
    use crate::ids::{AgentDefId, ProjectId, RunId, SessionId};

    use serde_json::json;
    use sqlx::{query, query_scalar};

    use super::{normalize, persist_normalized_events};
    use crate::{
        services::telemetry::{AcpAdapter, EventCollector},
        store::{backup::RetainedBackupConfig, Store},
    };

    #[tokio::test]
    async fn persists_each_normalized_event_with_its_run_identity_and_source_record() {
        let (container, _cleanup) =
            crate::testkit::postgres::start_postgres_named("locus-run-events-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &crate::testkit::postgres::NoopMigrationBackup,
                &RetainedBackupConfig::new(
                    "postgres://locus@localhost/locus",
                    "/var/lib/locus/artifacts",
                    "/var/lib/locus/backups",
                ),
            )
            .await
            .expect("run migrations");

        let project_id = ProjectId::generate();
        let agent_def_id = AgentDefId::generate();
        let session_id = SessionId::generate();
        let run_id = RunId::generate();
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'event persistence')")
            .bind(project_id)
            .execute(store.pool())
            .await
            .expect("insert project");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'event persistence', 1, '{}'::jsonb, '')",
        )
        .bind(agent_def_id)
        .execute(store.pool())
        .await
        .expect("insert agent definition");
        query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1, $2, $3, 'event persistence', 'agent/event-persistence')",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(agent_def_id)
        .execute(store.pool())
        .await
        .expect("insert session");
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
             VALUES ($1, $2, 'test-model', 'running')",
        )
        .bind(run_id)
        .bind(session_id)
        .execute(store.pool())
        .await
        .expect("insert run");

        let adapter = AcpAdapter;
        let first_raw = json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"first"}}}});
        let second_raw = json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"second"}}}});
        let captured = normalize(&adapter, [first_raw.clone(), second_raw.clone()])
            .expect("normalize source records");

        let conflicting_collector = EventCollector::new(2);
        let _ = conflicting_collector.capture(run_id, captured[0].clone());
        let conflicting_event = conflicting_collector.capture(run_id, captured[1].clone());
        store
            .persist_event(EventId::generate(), &conflicting_event)
            .await
            .expect("seed conflicting event");

        let error =
            persist_normalized_events(&store, &EventCollector::new(2), run_id, captured.clone())
                .await
                .expect_err("a conflicting event aborts the batch");
        assert!(error
            .to_string()
            .contains("persist normalized telemetry event"));
        let persisted_before_abort: i64 =
            query_scalar("SELECT count(*) FROM agents.events WHERE run_id = $1")
                .bind(run_id)
                .fetch_one(store.pool())
                .await
                .expect("count aborted batch events");
        assert_eq!(persisted_before_abort, 1);
        query("DELETE FROM agents.events WHERE run_id = $1")
            .bind(run_id)
            .execute(store.pool())
            .await
            .expect("clear conflict fixture");

        let persisted =
            persist_normalized_events(&store, &EventCollector::new(2), run_id, captured)
                .await
                .expect("persist normalized events");

        assert_eq!(persisted.len(), 2);
        let rows: serde_json::Value = query_scalar(
            "SELECT jsonb_agg(
                jsonb_build_object(
                    'run_id', run_id::text,
                    'seq', seq,
                    'raw', raw
                )
                ORDER BY seq
            )
            FROM agents.events
            WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_one(store.pool())
        .await
        .expect("read persisted events");
        assert_eq!(
            rows,
            json!([
                {
                    "run_id": run_id.to_string(),
                    "seq": 0,
                    "raw": first_raw,
                },
                {
                    "run_id": run_id.to_string(),
                    "seq": 1,
                    "raw": second_raw,
                }
            ])
        );

        // An ACP `session/update` carries no timestamp of its own, so `ts` is the store's.
        // It must still be present and well formed on every row.
        let timestamps: i64 =
            query_scalar("SELECT count(*) FROM agents.events WHERE run_id = $1 AND ts IS NOT NULL")
                .bind(run_id)
                .fetch_one(store.pool())
                .await
                .expect("count persisted timestamps");
        assert_eq!(timestamps, 2);
    }
}
