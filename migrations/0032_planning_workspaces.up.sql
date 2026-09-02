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

-- Existing bounded plans become resumable feature workspaces. The plan and revision
-- ids are reused as stable migration keys; requirement ids remain in the JSON state.
INSERT INTO core.planning_workspaces (id, project_id, scope, lifecycle, current_revision)
SELECT p.id, p.project_id, 'feature', 'in_progress', 1
FROM core.plans p
WHERE NOT EXISTS (
    SELECT 1 FROM core.planning_workspaces w WHERE w.id = p.id
);

INSERT INTO core.planning_workspace_revisions
    (id, workspace_id, revision, state, frozen_at, approved_at)
SELECT p.id,
       p.id,
       1,
       jsonb_build_object(
           'legacyPlanId', p.id,
           'title', p.title,
           'brief', p.goal,
           'legacyState', p.state,
           'requirements', COALESCE(
               (
                   SELECT jsonb_agg(
                       jsonb_build_object(
                           'id', r.requirement_id,
                           'body', r.body,
                           'changed', r.changed,
                           'carriesBoardCard', r.carries_board_card
                       ) ORDER BY r.requirement_id
                   )
                   FROM core.plan_requirements r
                   WHERE r.plan_id = p.id
               ),
               '[]'::jsonb
           )
       ),
       CASE WHEN p.state = 'approved'
            AND EXISTS (SELECT 1 FROM core.repos r WHERE r.project_id = p.project_id)
            THEN p.updated_at ELSE NULL END,
       CASE WHEN p.state = 'approved'
            AND EXISTS (SELECT 1 FROM core.repos r WHERE r.project_id = p.project_id)
            THEN p.updated_at ELSE NULL END
FROM core.plans p
WHERE NOT EXISTS (
    SELECT 1 FROM core.planning_workspace_revisions r WHERE r.id = p.id
);

INSERT INTO core.planning_workspace_specs (id, workspace_id, repo_id, name, state)
SELECT p.id,
       p.id,
       repo.id,
       'legacy-plan-' || p.id::text,
       jsonb_build_object(
           'legacyPlanId', p.id,
           'reviewed', p.state = 'approved',
           'requirements', COALESCE(
               (
                   SELECT jsonb_agg(
                       jsonb_build_object(
                           'id', r.requirement_id,
                           'body', r.body,
                           'taskIds', CASE WHEN r.carries_board_card THEN jsonb_build_array(r.requirement_id) ELSE '[]'::jsonb END,
                           'nonTaskOutcome', NOT r.carries_board_card
                       ) ORDER BY r.requirement_id
                   )
                   FROM core.plan_requirements r
                   WHERE r.plan_id = p.id
               ),
               '[]'::jsonb
           )
       )
FROM core.plans p
JOIN LATERAL (
    SELECT r.id
    FROM core.repos r
    WHERE r.project_id = p.project_id
    ORDER BY r.id
    LIMIT 1
) repo ON TRUE
WHERE NOT EXISTS (
    SELECT 1 FROM core.planning_workspace_specs s WHERE s.id = p.id
);

UPDATE core.planning_workspaces w
SET lifecycle = 'approved', approved_revision_id = w.id
FROM core.plans p
WHERE w.id = p.id
  AND p.state = 'approved'
  AND EXISTS (
      SELECT 1 FROM core.planning_workspace_specs s WHERE s.workspace_id = w.id
  );

INSERT INTO core.planning_workspace_events (id, workspace_id, revision_id, kind, payload)
SELECT p.id, p.id, p.id, 'backfilled', jsonb_build_object('legacyPlanId', p.id)
FROM core.plans p
WHERE NOT EXISTS (
    SELECT 1 FROM core.planning_workspace_events e WHERE e.id = p.id
);
