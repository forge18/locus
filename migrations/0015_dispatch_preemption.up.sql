ALTER TABLE core.dispatch_policy
ADD COLUMN preemption_enabled BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE agents.dispatch_preemptions (
    run_id UUID PRIMARY KEY REFERENCES agents.runs (id) ON DELETE CASCADE,
    handoff_context JSONB NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
