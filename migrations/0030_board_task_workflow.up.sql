ALTER TABLE board.tasks
ADD COLUMN workflow_def_id UUID REFERENCES workflows.workflow_defs (
    id
) ON DELETE SET NULL;

CREATE INDEX tasks_workflow_def_id_idx ON board.tasks (workflow_def_id);
