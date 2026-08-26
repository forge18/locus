-- Workflow domain log and its per-project position allocator.
-- Telemetry remains in agents.events; domain entries never use that table.
CREATE SCHEMA log;

CREATE TABLE log.project_streams (
    project_id UUID PRIMARY KEY REFERENCES core.projects (id) ON DELETE CASCADE,
    next_pos BIGINT NOT NULL DEFAULT 0 CHECK (next_pos >= 0)
);

CREATE TABLE log.entries (
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    stream_pos BIGINT NOT NULL CHECK (stream_pos > 0),
    kind TEXT NOT NULL CHECK (kind IN (
        'workflow.execution',
        'workflow.iteration',
        'workflow.guardrail_trip',
        'workflow.verify_result'
    )),
    v SMALLINT NOT NULL CHECK (v > 0),
    payload JSONB NOT NULL,
    actor TEXT NOT NULL CHECK (btrim(actor) <> ''),
    caused_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, stream_pos),
    FOREIGN KEY (project_id, caused_by)
    REFERENCES log.entries (project_id, stream_pos)
);

CREATE INDEX log_entries_project_kind_pos_idx
ON log.entries (project_id, kind, stream_pos);
CREATE INDEX log_entries_caused_by_idx
ON log.entries (project_id, caused_by);
