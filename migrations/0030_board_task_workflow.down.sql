DROP INDEX IF EXISTS board.tasks_workflow_def_id_idx;
ALTER TABLE board.tasks DROP COLUMN IF EXISTS workflow_def_id;
