ALTER TABLE agents.runs
ADD COLUMN permission_posture TEXT NOT NULL DEFAULT 'bypass'
CHECK (permission_posture IN ('bypass', 'gated'));

CREATE TABLE agents.checkpoints (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    ordinal BIGINT NOT NULL CHECK (ordinal >= 0),
    workspace JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (run_id, ordinal)
);

CREATE INDEX checkpoints_run_id_idx ON agents.checkpoints (run_id, ordinal);

CREATE TABLE agents.permission_gates (
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    seq BIGINT NOT NULL CHECK (seq >= 0),
    request_id TEXT NOT NULL CHECK (btrim(request_id) <> ''),
    diff JSONB,
    raw JSONB NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolution JSONB,
    PRIMARY KEY (run_id, seq)
);
