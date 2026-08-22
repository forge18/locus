CREATE SCHEMA workflows;

CREATE TABLE workflows.workflow_defs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    version INTEGER NOT NULL CHECK (version > 0),
    graph JSONB NOT NULL,
    spec JSONB NOT NULL,
    verify_command TEXT NOT NULL CHECK (btrim(verify_command) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT workflow_defs_project_name_version_key UNIQUE (project_id, name, version)
);

CREATE INDEX workflow_defs_project_id_idx ON workflows.workflow_defs (project_id);

CREATE FUNCTION workflows.reject_workflow_def_update() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'workflow definitions are immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER workflow_defs_are_immutable
BEFORE UPDATE ON workflows.workflow_defs
FOR EACH ROW EXECUTE FUNCTION workflows.reject_workflow_def_update();

CREATE TABLE workflows.schedules (
    id UUID PRIMARY KEY,
    workflow_def_id UUID NOT NULL REFERENCES workflows.workflow_defs (id),
    cron_expression TEXT NOT NULL CHECK (btrim(cron_expression) <> ''),
    paused_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX schedules_workflow_def_id_idx ON workflows.schedules (workflow_def_id);

CREATE TABLE workflows.executions (
    id UUID PRIMARY KEY,
    workflow_def_id UUID NOT NULL REFERENCES workflows.workflow_defs (id),
    schedule_id UUID REFERENCES workflows.schedules (id),
    status TEXT NOT NULL CHECK (status IN (
        'queued', 'running', 'completed', 'failed', 'skipped', 'cancelled'
    )),
    scheduled_for TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (status <> 'skipped' OR schedule_id IS NOT NULL)
);

CREATE INDEX executions_schedule_id_idx ON workflows.executions (schedule_id);
CREATE INDEX executions_workflow_def_id_idx ON workflows.executions (workflow_def_id);
CREATE UNIQUE INDEX executions_schedule_scheduled_for_key
    ON workflows.executions (schedule_id, scheduled_for)
    WHERE schedule_id IS NOT NULL AND scheduled_for IS NOT NULL;
CREATE UNIQUE INDEX executions_active_schedule_idx ON workflows.executions (schedule_id)
    WHERE status = 'running';

CREATE TABLE workflows.iterations (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflows.executions (id) ON DELETE CASCADE,
    run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    number INTEGER NOT NULL CHECK (number > 0),
    arbiter_class TEXT CHECK (arbiter_class IN ('bug', 'spec_gap', 'noise', 'ambiguity')),
    counts_toward_iteration_budget BOOLEAN NOT NULL DEFAULT true,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ,
    CONSTRAINT iterations_execution_number_key UNIQUE (execution_id, number),
    CHECK (arbiter_class <> 'noise' OR NOT counts_toward_iteration_budget)
);

CREATE INDEX iterations_run_id_idx ON workflows.iterations (run_id);

CREATE TABLE workflows.guardrail_trips (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflows.executions (id) ON DELETE CASCADE,
    iteration_id UUID REFERENCES workflows.iterations (id) ON DELETE SET NULL,
    run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    guardrail TEXT NOT NULL CHECK (btrim(guardrail) <> ''),
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    tripped_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX guardrail_trips_execution_id_idx ON workflows.guardrail_trips (execution_id);
CREATE INDEX guardrail_trips_iteration_id_idx ON workflows.guardrail_trips (iteration_id);

CREATE TABLE workflows.verify_results (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflows.executions (id) ON DELETE CASCADE,
    iteration_id UUID REFERENCES workflows.iterations (id) ON DELETE SET NULL,
    verify_node_id TEXT NOT NULL CHECK (btrim(verify_node_id) <> ''),
    command TEXT NOT NULL CHECK (btrim(command) <> ''),
    container_id TEXT NOT NULL CHECK (btrim(container_id) <> ''),
    exit_code INTEGER NOT NULL,
    passed BOOLEAN NOT NULL,
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((passed AND exit_code = 0) OR (NOT passed AND exit_code <> 0))
);

CREATE INDEX verify_results_execution_id_idx ON workflows.verify_results (execution_id);
CREATE INDEX verify_results_iteration_id_idx ON workflows.verify_results (iteration_id);
