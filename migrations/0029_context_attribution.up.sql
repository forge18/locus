-- Read-only attribution over the existing telemetry, workflow verification, and memory rows.
-- A materialization snapshot is taken from the run's SessionStart raw/payload record; no new
-- event shape or backfill is required.
CREATE VIEW agents.context_attribution AS
WITH run_snapshots AS (
    SELECT DISTINCT ON (run_id)
        run_id,
        COALESCE(
            raw -> 'materialization_snapshot',
            raw -> 'context_snapshot',
            payload -> 'materialization_snapshot',
            payload -> 'context_snapshot',
            '{}'::jsonb
        ) AS materialization_snapshot
    FROM agents.events
    WHERE verb = 'session_start'
    ORDER BY run_id, seq
),
context_events AS (
    SELECT
        e.run_id,
        e.id,
        e.seq,
        COALESCE(
            e.payload ->> 'context_event',
            e.raw ->> 'context_event',
            CASE
                WHEN e.payload ? 'injection' OR e.raw ? 'injection' THEN 'injection'
                WHEN e.payload ? 'recall' OR e.raw ? 'recall' THEN 'recall'
            END
        ) AS context_event,
        COALESCE(
            e.payload ->> 'memory_id',
            e.payload #>> '{injection,memory_id}',
            e.payload #>> '{recall,memory_id}',
            e.raw ->> 'memory_id',
            e.raw #>> '{injection,memory_id}',
            e.raw #>> '{recall,memory_id}'
        ) AS context_memory_id
    FROM agents.events e
    WHERE e.payload ? 'context_event'
       OR e.raw ? 'context_event'
       OR e.payload ? 'injection'
       OR e.raw ? 'injection'
       OR e.payload ? 'recall'
       OR e.raw ? 'recall'
       OR e.payload ? 'memory_id'
       OR e.raw ? 'memory_id'
    UNION ALL
    SELECT
        rf.run_id,
        NULL::UUID AS id,
        NULL::BIGINT AS seq,
        'recall' AS context_event,
        rf.fact_id::text AS context_memory_id
    FROM memory.retrieval_feedback rf
),
verification AS (
    SELECT DISTINCT ON (i.run_id)
        i.run_id,
        vr.id AS verify_result_id,
        vr.passed AS verify_passed,
        vr.exit_code AS verify_exit_code,
        EXTRACT(EPOCH FROM (vr.completed_at - i.started_at)) * 1000 AS verification_duration_ms
    FROM workflows.verify_results vr
    JOIN workflows.iterations i ON i.id = vr.iteration_id
    WHERE i.run_id IS NOT NULL
    ORDER BY i.run_id, vr.completed_at, vr.id
),
verification_events AS (
    SELECT
        e.run_id,
        MAX(
            CASE
                WHEN COALESCE(e.payload ->> 'duration_ms', e.raw ->> 'duration_ms') ~ '^[0-9]+$'
                THEN COALESCE(e.payload ->> 'duration_ms', e.raw ->> 'duration_ms')::BIGINT
            END
        ) AS event_duration_ms,
        SUM(
            CASE
                WHEN COALESCE(e.payload #>> '{usage,input}', e.raw #>> '{usage,input}') ~ '^[0-9]+$'
                THEN COALESCE(e.payload #>> '{usage,input}', e.raw #>> '{usage,input}')::BIGINT
                ELSE 0
            END
            + CASE
                WHEN COALESCE(e.payload #>> '{usage,output}', e.raw #>> '{usage,output}') ~ '^[0-9]+$'
                THEN COALESCE(e.payload #>> '{usage,output}', e.raw #>> '{usage,output}')::BIGINT
                ELSE 0
            END
        ) AS event_tokens
    FROM agents.events e
    WHERE e.payload ->> 'tool' ILIKE '%verify%'
       OR e.raw ->> 'tool' ILIKE '%verify%'
       OR e.payload ? 'verify_command'
       OR e.raw ? 'verify_command'
    GROUP BY e.run_id
)
SELECT
    session_row.project_id,
    c.run_id,
    r.session_id,
    c.id AS event_id,
    c.seq AS event_seq,
    c.context_event,
    c.context_memory_id AS memory_id,
    m.path AS memory_path,
    s.materialization_snapshot,
    s.materialization_snapshot -> 'base_context' AS base_context,
    s.materialization_snapshot -> 'rules' AS rules,
    s.materialization_snapshot -> 'skills' AS skills,
    v.verify_result_id,
    v.verify_passed,
    v.verify_exit_code,
    COALESCE(ve.event_duration_ms, v.verification_duration_ms) AS verification_duration_ms,
    ve.event_tokens AS verification_tokens,
    tool.event_id AS tool_result_event_id,
    tool.tool,
    tool.payload_bytes AS tool_result_payload_bytes
FROM context_events c
JOIN agents.runs r ON r.id = c.run_id
JOIN agents.sessions session_row ON session_row.id = r.session_id
LEFT JOIN run_snapshots s ON s.run_id = c.run_id
LEFT JOIN memory.store m
    ON m.id::text = c.context_memory_id
   AND m.project_id = session_row.project_id
LEFT JOIN verification v ON v.run_id = c.run_id
LEFT JOIN verification_events ve ON ve.run_id = c.run_id
LEFT JOIN LATERAL (
    SELECT
        tr.id AS event_id,
        tr.payload ->> 'tool' AS tool,
        length(COALESCE(tr.payload ->> 'text', '')) AS payload_bytes
    FROM agents.events tr
    WHERE tr.run_id = c.run_id
      AND tr.verb = 'tool_result'
      AND tr.seq >= c.seq
    ORDER BY tr.seq, tr.id
    LIMIT 1
) tool ON TRUE;
