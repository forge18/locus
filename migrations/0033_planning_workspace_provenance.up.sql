CREATE TABLE core.planning_workspace_task_provenance (
    id UUID PRIMARY KEY,
    materialization_id UUID NOT NULL
    REFERENCES core.planning_workspace_materializations (id) ON DELETE RESTRICT,
    workspace_id UUID NOT NULL
    REFERENCES core.planning_workspaces (id) ON DELETE RESTRICT,
    revision_id UUID NOT NULL
    REFERENCES core.planning_workspace_revisions (id) ON DELETE RESTRICT,
    board_task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE RESTRICT,
    spec_id UUID NOT NULL REFERENCES core.planning_workspace_specs (
        id
    ) ON DELETE RESTRICT,
    requirement_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (materialization_id, board_task_id, spec_id, requirement_id)
);

CREATE INDEX planning_workspace_task_provenance_task_idx
ON core.planning_workspace_task_provenance (board_task_id);
