CREATE SCHEMA bots;

ALTER TABLE agents.runs
    ADD COLUMN IF NOT EXISTS agent_def_id UUID REFERENCES agents.agent_defs (id);

UPDATE agents.runs AS runs
SET agent_def_id = sessions.agent_def_id
FROM agents.sessions AS sessions
WHERE runs.session_id = sessions.id
  AND runs.agent_def_id IS NULL;

CREATE INDEX runs_agent_def_id_idx ON agents.runs (agent_def_id);

CREATE TABLE bots.bots (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    agent_def_id UUID NOT NULL REFERENCES agents.agent_defs (id),
    home_session_id UUID NOT NULL UNIQUE REFERENCES agents.sessions (id) ON DELETE CASCADE,
    branch TEXT NOT NULL CHECK (
        btrim(branch) <> ''
        AND branch LIKE 'bots/%'
        AND branch NOT IN ('main', 'master')
    ),
    container_id TEXT UNIQUE,
    container_state TEXT NOT NULL DEFAULT 'cold'
        CHECK (container_state IN ('cold', 'running', 'warm')),
    warm_until TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ,
    total_cost_micros BIGINT CHECK (total_cost_micros IS NULL OR total_cost_micros >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, name)
);

CREATE INDEX bots_project_id_idx ON bots.bots (project_id);
CREATE INDEX bots_home_session_id_idx ON bots.bots (home_session_id);
CREATE INDEX bots_container_state_idx ON bots.bots (container_state);

CREATE TABLE bots.routines (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL REFERENCES bots.bots (id) ON DELETE CASCADE,
    prompt TEXT NOT NULL CHECK (btrim(prompt) <> ''),
    cron_expression TEXT NOT NULL CHECK (btrim(cron_expression) <> ''),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    skipped_count INTEGER NOT NULL DEFAULT 0 CHECK (skipped_count >= 0),
    schedule_id UUID UNIQUE REFERENCES workflows.schedules (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX routines_bot_id_idx ON bots.routines (bot_id);
CREATE INDEX routines_enabled_idx ON bots.routines (enabled);

CREATE TABLE bots.routine_executions (
    id UUID PRIMARY KEY,
    routine_id UUID REFERENCES bots.routines (id) ON DELETE SET NULL,
    bot_id UUID NOT NULL REFERENCES bots.bots (id) ON DELETE CASCADE,
    prompt TEXT NOT NULL CHECK (btrim(prompt) <> ''),
    scheduled_for TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'skipped')),
    result JSONB,
    run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    attribution TEXT NOT NULL CHECK (attribution IN ('routine-fired', 'test-run')),
    test_run BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ
);

CREATE INDEX routine_executions_routine_id_idx ON bots.routine_executions (routine_id);
CREATE INDEX routine_executions_bot_id_idx ON bots.routine_executions (bot_id);
CREATE INDEX routine_executions_active_idx
    ON bots.routine_executions (routine_id)
    WHERE status = 'running' AND NOT test_run;
