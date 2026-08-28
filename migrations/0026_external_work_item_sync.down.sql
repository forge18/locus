DROP INDEX IF EXISTS board.external_sync_changes_task_time_idx;
DROP INDEX IF EXISTS board.task_comments_task_id_idx;
DROP INDEX IF EXISTS board.external_work_items_sync_cursor_idx;

DROP TABLE IF EXISTS board.external_sync_changes;
DROP TABLE IF EXISTS board.task_comments;

ALTER TABLE board.task_transitions
    DROP CONSTRAINT IF EXISTS task_transitions_actor_kind_check,
    DROP COLUMN IF EXISTS evidence,
    DROP COLUMN IF EXISTS actor_label;
ALTER TABLE board.task_transitions
    ADD CONSTRAINT task_transitions_actor_kind_check
    CHECK (actor_kind IN ('human', 'agent', 'system'));

ALTER TABLE board.external_work_item_providers
    DROP CONSTRAINT IF EXISTS external_work_item_providers_sync_interval_check,
    DROP COLUMN IF EXISTS sync_interval_seconds;

ALTER TABLE board.external_work_items
DROP COLUMN IF EXISTS last_conflict_reason,
DROP COLUMN IF EXISTS last_conflict_winner,
DROP COLUMN IF EXISTS unmapped_external_status,
DROP COLUMN IF EXISTS last_synced_at,
DROP COLUMN IF EXISTS last_sync_error,
DROP COLUMN IF EXISTS last_external_status_at,
DROP COLUMN IF EXISTS last_local_status_at,
DROP COLUMN IF EXISTS note_watermark,
DROP COLUMN IF EXISTS last_pushed_status,
DROP COLUMN IF EXISTS pull_cursor;
