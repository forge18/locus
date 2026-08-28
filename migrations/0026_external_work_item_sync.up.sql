ALTER TABLE board.external_work_item_providers
    ADD COLUMN IF NOT EXISTS sync_interval_seconds INTEGER NOT NULL DEFAULT 60;

ALTER TABLE board.external_work_item_providers
    DROP CONSTRAINT IF EXISTS external_work_item_providers_sync_interval_check;
ALTER TABLE board.external_work_item_providers
    ADD CONSTRAINT external_work_item_providers_sync_interval_check
    CHECK (sync_interval_seconds BETWEEN 1 AND 86400);

ALTER TABLE board.external_work_items
    ADD COLUMN IF NOT EXISTS pull_cursor TEXT,
    ADD COLUMN IF NOT EXISTS last_pushed_status TEXT,
    ADD COLUMN IF NOT EXISTS note_watermark TEXT,
    ADD COLUMN IF NOT EXISTS last_local_status_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_external_status_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_sync_error TEXT,
    ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS unmapped_external_status TEXT,
    ADD COLUMN IF NOT EXISTS last_conflict_winner TEXT,
    ADD COLUMN IF NOT EXISTS last_conflict_reason TEXT;

CREATE INDEX IF NOT EXISTS external_work_items_sync_cursor_idx
    ON board.external_work_items (pull_cursor)
    WHERE pull_cursor IS NOT NULL;

ALTER TABLE board.task_transitions
    ADD COLUMN IF NOT EXISTS actor_label TEXT,
    ADD COLUMN IF NOT EXISTS evidence JSONB NOT NULL DEFAULT '[]'::JSONB;

ALTER TABLE board.task_transitions
    DROP CONSTRAINT IF EXISTS task_transitions_actor_kind_check;
ALTER TABLE board.task_transitions
    ADD CONSTRAINT task_transitions_actor_kind_check
    CHECK (actor_kind IN ('human', 'agent', 'system', 'sync'));

CREATE TABLE IF NOT EXISTS board.task_comments (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    author TEXT NOT NULL CHECK (btrim(author) <> ''),
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    origin TEXT NOT NULL CHECK (origin IN ('local', 'external')),
    external_provider TEXT,
    external_note_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (task_id, origin, external_provider, external_note_id)
);

CREATE INDEX IF NOT EXISTS task_comments_task_id_idx
    ON board.task_comments (task_id, created_at, id);

CREATE TABLE IF NOT EXISTS board.external_sync_changes (
    task_id UUID NOT NULL REFERENCES board.tasks (id) ON DELETE CASCADE,
    change_id TEXT NOT NULL CHECK (btrim(change_id) <> ''),
    change_kind TEXT NOT NULL CHECK (change_kind IN ('status', 'note')),
    occurred_at TIMESTAMPTZ NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (task_id, change_kind, change_id)
);

CREATE INDEX IF NOT EXISTS external_sync_changes_task_time_idx
    ON board.external_sync_changes (task_id, occurred_at, change_id);
