CREATE TABLE agents.turns (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    ordinal BIGINT NOT NULL CHECK (ordinal >= 0),
    prompt_event_id UUID NOT NULL REFERENCES agents.events (id),
    response_event_id UUID REFERENCES agents.events (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    UNIQUE (run_id, ordinal),
    CHECK (response_event_id IS NULL OR response_event_id <> prompt_event_id)
);
