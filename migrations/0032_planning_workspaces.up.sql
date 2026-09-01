-- Durable Planning Workspace wrapper. Bounded core.plans remain the child-spec projection.
CREATE TABLE core.planning_workspaces (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    scope TEXT NOT NULL CHECK (scope IN ('amendment', 'feature', 'project')),
    lifecycle TEXT NOT NULL DEFAULT 'draft' CHECK (
        lifecycle IN ('draft', 'in_progress', 'ready_for_approval', 'approved', 'deleted')
    ),
    current_revision INTEGER NOT NULL DEFAULT 1 CHECK (current_revision > 0),
    approved_revision_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((lifecycle = 'approved') = (approved_revision_id IS NOT NULL))
);

CREATE INDEX planning_workspaces_project_updated_idx
    ON core.planning_workspaces (project_id, updated_at DESC);

CREATE TABLE core.planning_workspace_revisions (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES core.planning_workspaces (id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    state JSONB NOT NULL,
    frozen_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, revision),
    CHECK (approved_at IS NULL OR frozen_at IS NOT NULL)
);

ALTER TABLE core.planning_workspaces
    ADD CONSTRAINT planning_workspaces_approved_revision_fk
    FOREIGN KEY (approved_revision_id)
    REFERENCES core.planning_workspace_revisions (id);

CREATE TABLE core.planning_workspace_specs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES core.planning_workspaces (id) ON DELETE CASCADE,
    repo_id UUID NOT NULL REFERENCES core.repos (id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    state JSONB NOT NULL DEFAULT '{}'::jsonb,
    stale BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, name)
);

CREATE INDEX planning_workspace_specs_workspace_idx
    ON core.planning_workspace_specs (workspace_id);

CREATE TABLE core.planning_workspace_sessions (
    workspace_id UUID NOT NULL REFERENCES core.planning_workspaces (id) ON DELETE CASCADE,
    spec_id UUID REFERENCES core.planning_workspace_specs (id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES agents.sessions (id) ON DELETE CASCADE,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, session_id)
);

CREATE TABLE core.planning_workspace_events (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES core.planning_workspaces (id) ON DELETE CASCADE,
    revision_id UUID REFERENCES core.planning_workspace_revisions (id) ON DELETE SET NULL,
    kind TEXT NOT NULL CHECK (btrim(kind) <> ''),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX planning_workspace_events_workspace_created_idx
    ON core.planning_workspace_events (workspace_id, created_at, id);

CREATE TABLE core.planning_workspace_materializations (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES core.planning_workspaces (id) ON DELETE RESTRICT,
    revision_id UUID NOT NULL REFERENCES core.planning_workspace_revisions (id) ON DELETE RESTRICT,
    board_task_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, revision_id)
);
