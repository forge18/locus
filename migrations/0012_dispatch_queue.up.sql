CREATE TABLE core.dispatch_policy (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    global_parallelism INTEGER NOT NULL CHECK (global_parallelism > 0),
    per_project_parallelism INTEGER NOT NULL CHECK (
        per_project_parallelism > 0
    ),
    priority_method TEXT NOT NULL CHECK (priority_method IN (
        'plan_order', 'manual', 'unblocks_most', 'shortest_first'
    )),
    tie_break TEXT NOT NULL CHECK (tie_break = 'longest_waiting')
);

INSERT INTO core.dispatch_policy (
    singleton,
    global_parallelism,
    per_project_parallelism,
    priority_method,
    tie_break
) VALUES (TRUE, 6, 3, 'plan_order', 'longest_waiting');

CREATE TABLE agents.dispatch_queue (
    run_id UUID PRIMARY KEY REFERENCES agents.runs (id) ON DELETE CASCADE,
    plan_order BIGINT NOT NULL,
    manual_order BIGINT NOT NULL,
    unblocks_count INTEGER NOT NULL CHECK (unblocks_count >= 0),
    estimate_minutes INTEGER NOT NULL CHECK (estimate_minutes >= 0),
    enqueued_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX dispatch_queue_enqueued_at_idx ON agents.dispatch_queue (
    enqueued_at
);
