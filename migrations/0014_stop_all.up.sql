CREATE TABLE core.project_autorun (
    project_id UUID PRIMARY KEY REFERENCES core.projects (id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.stop_all_snapshots (
    id UUID PRIMARY KEY,
    stopped_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    restore_expires_at TIMESTAMPTZ NOT NULL,
    restored_at TIMESTAMPTZ,
    CHECK (restore_expires_at = stopped_at + INTERVAL '10 minutes'),
    CHECK (restored_at IS NULL OR restored_at <= restore_expires_at)
);

CREATE TABLE core.stop_all_run_snapshots (
    snapshot_id UUID NOT NULL REFERENCES core.stop_all_snapshots (id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    prior_status TEXT NOT NULL CHECK (prior_status IN ('queued', 'running')),
    PRIMARY KEY (snapshot_id, run_id)
);

CREATE TABLE core.stop_all_autorun_snapshots (
    snapshot_id UUID NOT NULL REFERENCES core.stop_all_snapshots (id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL,
    PRIMARY KEY (snapshot_id, project_id)
);

CREATE TABLE core.stop_all_schedule_snapshots (
    snapshot_id UUID NOT NULL REFERENCES core.stop_all_snapshots (id) ON DELETE CASCADE,
    schedule_id UUID NOT NULL REFERENCES workflows.schedules (id) ON DELETE CASCADE,
    paused_at TIMESTAMPTZ,
    PRIMARY KEY (snapshot_id, schedule_id)
);

ALTER TABLE agents.runs DROP CONSTRAINT runs_status_check;
ALTER TABLE agents.runs ADD CONSTRAINT runs_status_check CHECK (
    status IN (
        'queued', 'running', 'paused', 'stopped', 'completed', 'aborted', 'cancelled'
    )
);
