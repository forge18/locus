-- M0.7 current desktop reconciliation.
-- This migration adds projection state without replacing the append-only source tables.

ALTER TABLE core.projects
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;

ALTER TABLE core.repos
    ADD CONSTRAINT repos_working_copy_path_global_key UNIQUE (working_copy_path);

CREATE TABLE core.repo_project_history (
    id UUID PRIMARY KEY,
    repo_id UUID NOT NULL REFERENCES core.repos (id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX repo_project_history_repo_idx ON core.repo_project_history (repo_id, recorded_at);

ALTER TABLE core.project_autorun
    ADD COLUMN IF NOT EXISTS state TEXT NOT NULL DEFAULT 'off'
        CHECK (state IN ('on', 'off', 'suspended'));
ALTER TABLE core.stop_all_autorun_snapshots
    ADD COLUMN IF NOT EXISTS state TEXT NOT NULL DEFAULT 'off'
        CHECK (state IN ('on', 'off', 'suspended'));
CREATE TABLE core.stop_all_handoffs (
    snapshot_id UUID NOT NULL REFERENCES core.stop_all_snapshots (id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    payload JSONB NOT NULL,
    PRIMARY KEY (snapshot_id, run_id)
);
-- Existing Stop all snapshots predate the tri-state column; retain their saved posture.
UPDATE core.stop_all_autorun_snapshots
SET state = CASE WHEN enabled THEN 'on' ELSE 'off' END;
UPDATE core.project_autorun SET state = CASE WHEN enabled THEN 'on' ELSE 'off' END;

CREATE TABLE core.project_autorun_policy (
    project_id UUID PRIMARY KEY REFERENCES core.projects (id) ON DELETE CASCADE,
    review_pause_threshold INTEGER NOT NULL CHECK (review_pause_threshold >= 0),
    inbox_budget_per_hour INTEGER NOT NULL CHECK (inbox_budget_per_hour >= 0),
    change_lines_ceiling INTEGER CHECK (change_lines_ceiling IS NULL OR change_lines_ceiling >= 0),
    change_files_ceiling INTEGER CHECK (change_files_ceiling IS NULL OR change_files_ceiling >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.guardrail_defaults (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    max_iterations INTEGER NOT NULL CHECK (max_iterations > 0),
    token_budget BIGINT CHECK (token_budget IS NULL OR token_budget > 0),
    stuck_iterations INTEGER NOT NULL CHECK (stuck_iterations > 0),
    kill_and_reassign BOOLEAN NOT NULL,
    change_lines_ceiling INTEGER CHECK (change_lines_ceiling IS NULL OR change_lines_ceiling >= 0),
    change_files_ceiling INTEGER CHECK (change_files_ceiling IS NULL OR change_files_ceiling >= 0),
    network_tier TEXT NOT NULL,
    block_system_changes BOOLEAN NOT NULL,
    autopilot BOOLEAN NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO core.guardrail_defaults
    (singleton, max_iterations, token_budget, stuck_iterations, kill_and_reassign,
     change_lines_ceiling, change_files_ceiling, network_tier, block_system_changes, autopilot)
VALUES (TRUE, 8, NULL, 3, TRUE, NULL, NULL, 'open', TRUE, FALSE)
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE core.qa_check_runs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    check_source_id TEXT NOT NULL CHECK (btrim(check_source_id) <> ''),
    trigger TEXT NOT NULL CHECK (trigger IN ('manual', 'push', 'hourly', 'daily')),
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    skipped_at TIMESTAMPTZ,
    CONSTRAINT qa_check_runs_terminal_state_check
        CHECK (finished_at IS NULL OR skipped_at IS NULL),
    UNIQUE (id, project_id, check_source_id)
);
CREATE INDEX qa_check_runs_project_source_idx ON core.qa_check_runs (project_id, check_source_id, started_at DESC);
CREATE UNIQUE INDEX qa_check_runs_active_project_source_key
    ON core.qa_check_runs (project_id, check_source_id)
    WHERE finished_at IS NULL AND skipped_at IS NULL;

CREATE TABLE core.qa_findings (
    id UUID PRIMARY KEY,
    check_run_id UUID NOT NULL,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    check_source_id TEXT NOT NULL CHECK (btrim(check_source_id) <> ''),
    severity TEXT NOT NULL CHECK (severity IN ('fail', 'warn')),
    title TEXT NOT NULL CHECK (btrim(title) <> ''),
    location TEXT NOT NULL CHECK (btrim(location) <> ''),
    explanation TEXT NOT NULL CHECK (btrim(explanation) <> ''),
    sent_to_inbox BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (check_run_id, project_id, check_source_id)
        REFERENCES core.qa_check_runs (id, project_id, check_source_id) ON DELETE CASCADE
);
CREATE INDEX qa_findings_project_source_idx ON core.qa_findings (project_id, check_source_id);

CREATE TABLE core.qa_schedules (
    project_id UUID PRIMARY KEY REFERENCES core.projects (id) ON DELETE CASCADE,
    schedule TEXT NOT NULL CHECK (schedule IN ('manual', 'push', 'hourly', 'daily')) DEFAULT 'manual',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE agents.sessions
    ADD COLUMN IF NOT EXISTS interact_state TEXT NOT NULL DEFAULT 'open'
        CHECK (interact_state IN ('open', 'promoted', 'discarded'));
UPDATE agents.sessions SET interact_state = CASE WHEN board_task_id IS NULL THEN 'open' ELSE 'promoted' END WHERE interact_state = 'open';
CREATE INDEX sessions_interact_state_idx ON agents.sessions (project_id, interact_state);

ALTER TABLE agents.runs
    ADD COLUMN IF NOT EXISTS failed_iterations INTEGER NOT NULL DEFAULT 0 CHECK (failed_iterations >= 0),
    ADD COLUMN IF NOT EXISTS verify_status TEXT CHECK (verify_status IN ('running', 'passed', 'failed', 'waiting_gate', 'na', 'aborted')),
    ADD COLUMN IF NOT EXISTS resolved_guardrails JSONB;

ALTER TABLE memory.store
    ADD COLUMN IF NOT EXISTS confidence_state TEXT NOT NULL DEFAULT 'asserted'
        CHECK (confidence_state IN ('verified', 'asserted', 'decaying', 'contradicted')),
    ADD COLUMN IF NOT EXISTS current_revision INTEGER NOT NULL DEFAULT 1 CHECK (current_revision > 0);

CREATE TABLE memory.fact_revisions (
    fact_id UUID NOT NULL REFERENCES memory.store (id) ON DELETE CASCADE,
    rev INTEGER NOT NULL CHECK (rev > 0),
    value TEXT NOT NULL CHECK (btrim(value) <> ''),
    written_by_run UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    curated_by TEXT,
    score DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (fact_id, rev),
    CHECK (score IS NULL OR score BETWEEN 0 AND 1)
);
INSERT INTO memory.fact_revisions
    (fact_id, rev, value, written_by_run, curated_by, score, created_at)
SELECT id, 1, body, source_run_id, NULL, NULL, created_at
FROM memory.store;

CREATE TABLE memory.retrieval_feedback (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    fact_id UUID REFERENCES memory.store (id) ON DELETE SET NULL,
    useful BOOLEAN,
    changed_answer BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE workflows.schedules
    ALTER COLUMN workflow_def_id DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS run_mode TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (run_mode IN ('once', 'scheduled', 'hold')),
    ADD COLUMN IF NOT EXISTS agent_def_id UUID REFERENCES agents.agent_defs (id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS harness TEXT,
    ADD COLUMN IF NOT EXISTS project_id UUID REFERENCES core.projects (id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS spec_id UUID,
    ADD COLUMN IF NOT EXISTS prompt TEXT,
    ADD COLUMN IF NOT EXISTS guardrail_overrides JSONB NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE core.providers
    ADD COLUMN IF NOT EXISTS verification_expires_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS verification_stale_after INTERVAL;

CREATE TABLE core.plans (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (btrim(title) <> ''),
    goal TEXT NOT NULL CHECK (btrim(goal) <> ''),
    stage TEXT NOT NULL CHECK (stage IN ('inputs', 'orient', 'converse', 'synthesis', 'recommend', 'decompose', 'approved')) DEFAULT 'inputs',
    state TEXT NOT NULL CHECK (state IN ('in_progress', 'draft_rejected', 'approved')) DEFAULT 'in_progress',
    confidence DOUBLE PRECISION CHECK (confidence IS NULL OR confidence BETWEEN 0 AND 1),
    open_count INTEGER NOT NULL DEFAULT 0 CHECK (open_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE core.plan_requirements (
    plan_id UUID NOT NULL REFERENCES core.plans (id) ON DELETE CASCADE,
    requirement_id TEXT NOT NULL CHECK (btrim(requirement_id) <> ''),
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    changed BOOLEAN NOT NULL DEFAULT FALSE,
    carries_board_card BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (plan_id, requirement_id)
);
CREATE TABLE core.plan_stage_history (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES core.plans (id) ON DELETE CASCADE,
    stage TEXT NOT NULL CHECK (stage IN ('inputs', 'orient', 'converse', 'synthesis', 'recommend', 'decompose', 'approved')),
    description TEXT NOT NULL DEFAULT '',
    duration_seconds BIGINT CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.harness_adapter_configs (
    harness TEXT PRIMARY KEY CHECK (btrim(harness) <> ''),
    adapter_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE agents.runs
    DROP CONSTRAINT IF EXISTS runs_routing_decision_check;
ALTER TABLE agents.runs
    ADD CONSTRAINT runs_routing_decision_check CHECK (
        (
            routing_requested_band IS NULL
            AND routing_selected_band IS NULL
            AND routing_effort IS NULL
            AND routing_approval_required IS NULL
        )
        OR (
            routing_requested_band IN (
                'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
            )
            AND (
                routing_selected_band IS NULL
                OR routing_selected_band IN (
                    'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
                )
            )
            AND routing_effort IS NOT NULL
            AND btrim(routing_effort) <> ''
            AND routing_effort IN ('low', 'medium', 'high', 'xhigh')
            AND routing_approval_required IS NOT NULL
        )
    );

ALTER TABLE workflows.schedules
    ADD CONSTRAINT schedules_custom_shape_check CHECK (
        run_mode <> 'once' OR (workflow_def_id IS NOT NULL OR prompt IS NOT NULL OR spec_id IS NOT NULL)
    );
