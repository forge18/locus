CREATE SCHEMA board;

CREATE TABLE board.tasks (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    repo_id UUID REFERENCES core.repos (id) ON DELETE SET NULL,
    summary TEXT NOT NULL CHECK (btrim(summary) <> ''),
    description TEXT NOT NULL DEFAULT '',
    column_name TEXT NOT NULL CHECK (column_name IN (
        'ready', 'in_progress', 'testing', 'reviewing', 'waiting_for_approval', 'done'
    )) DEFAULT 'ready',
    blocked BOOLEAN NOT NULL DEFAULT false,
    blocked_reason TEXT,
    blocked_clear_condition TEXT,
    assigned_agent_def_id UUID REFERENCES agents.agent_defs (id) ON DELETE SET NULL,
    session_id UUID REFERENCES agents.sessions (id) ON DELETE SET NULL,
    verify_command TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((blocked AND blocked_reason IS NOT NULL AND blocked_clear_condition IS NOT NULL) OR NOT blocked)
);

CREATE INDEX tasks_project_id_idx ON board.tasks (project_id);
CREATE INDEX tasks_repo_id_idx ON board.tasks (repo_id);
CREATE INDEX tasks_column_name_idx ON board.tasks (project_id, column_name);

CREATE TABLE board.task_dependencies (
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    blocked_by_task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    workflow_node_id TEXT NOT NULL CHECK (btrim(workflow_node_id) <> ''),
    PRIMARY KEY (task_id, blocked_by_task_id),
    CHECK (task_id <> blocked_by_task_id)
);

CREATE INDEX task_dependencies_blocked_by_idx ON board.task_dependencies (blocked_by_task_id);

CREATE TABLE board.task_transitions (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    from_column TEXT CHECK (from_column IN (
        'ready', 'in_progress', 'testing', 'reviewing', 'waiting_for_approval', 'done'
    )),
    to_column TEXT NOT NULL CHECK (to_column IN (
        'ready', 'in_progress', 'testing', 'reviewing', 'waiting_for_approval', 'done'
    )),
    actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'agent', 'system')),
    actor_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX task_transitions_task_id_idx ON board.task_transitions (task_id, created_at);

CREATE TABLE board.task_assignments (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    agent_def_id UUID NOT NULL REFERENCES agents.agent_defs (id),
    assigned_by_kind TEXT NOT NULL CHECK (assigned_by_kind IN ('human', 'agent', 'system')),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    unassigned_at TIMESTAMPTZ
);

CREATE INDEX task_assignments_task_id_idx ON board.task_assignments (task_id, assigned_at);

CREATE TABLE board.task_runs (
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (task_id, run_id)
);

CREATE TABLE board.task_evidence (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    event_id UUID REFERENCES agents.events (id) ON DELETE SET NULL,
    summary TEXT NOT NULL CHECK (btrim(summary) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX task_evidence_task_id_idx ON board.task_evidence (task_id);

CREATE TABLE board.github_issues (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL UNIQUE REFERENCES board.tasks (id) ON DELETE CASCADE,
    repository TEXT NOT NULL CHECK (btrim(repository) <> ''),
    issue_number BIGINT NOT NULL CHECK (issue_number > 0),
    url TEXT NOT NULL CHECK (btrim(url) <> ''),
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    labels JSONB NOT NULL DEFAULT '[]'::jsonb,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (repository, issue_number)
);
