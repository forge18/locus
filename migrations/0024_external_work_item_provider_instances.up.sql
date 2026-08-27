-- Upgrade the already-shipped M7 schema to support immutable provider instances and
-- the durable workflow/completion fields added by the plugin-backed integration.

ALTER TABLE board.external_work_item_providers
    DROP CONSTRAINT IF EXISTS external_work_item_providers_pkey;
ALTER TABLE board.external_work_item_providers
    ADD CONSTRAINT external_work_item_providers_pkey
    PRIMARY KEY (plugin_id, host, provider_project);

ALTER TABLE board.external_work_items
    ADD COLUMN IF NOT EXISTS workflow_def_id UUID;

UPDATE board.external_work_items AS external_item
SET workflow_def_id = latest.id
FROM board.tasks AS task
JOIN LATERAL (
    SELECT workflow.id
    FROM workflows.workflow_defs AS workflow
    WHERE workflow.project_id = task.project_id
    ORDER BY workflow.created_at DESC, workflow.id DESC
    LIMIT 1
) AS latest ON TRUE
WHERE external_item.task_id = task.id
  AND external_item.workflow_def_id IS NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM board.external_work_items
        WHERE workflow_def_id IS NULL
    ) THEN
        RAISE EXCEPTION
            'cannot upgrade external work items without a project workflow definition';
    END IF;
END;
$$;

ALTER TABLE board.external_work_items
    ALTER COLUMN workflow_def_id SET NOT NULL;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'external_work_items_workflow_def_id_fkey'
          AND conrelid = 'board.external_work_items'::regclass
    ) THEN
        ALTER TABLE board.external_work_items
            ADD CONSTRAINT external_work_items_workflow_def_id_fkey
            FOREIGN KEY (workflow_def_id) REFERENCES workflows.workflow_defs (id);
    END IF;
END;
$$;

ALTER TABLE board.external_completion_outbox
    ADD COLUMN IF NOT EXISTS locator TEXT;

UPDATE board.external_completion_outbox AS completion
SET locator = format('locus://%s/task/%s', task.project_id, task.id)
FROM board.tasks AS task
WHERE completion.task_id = task.id
  AND completion.locator IS NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM board.external_completion_outbox
        WHERE locator IS NULL OR btrim(locator) = ''
    ) THEN
        RAISE EXCEPTION 'cannot upgrade external completions without a task locator';
    END IF;
END;
$$;

ALTER TABLE board.external_completion_outbox
    ALTER COLUMN locator SET NOT NULL;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'external_completion_outbox_locator_check'
          AND conrelid = 'board.external_completion_outbox'::regclass
    ) THEN
        ALTER TABLE board.external_completion_outbox
            ADD CONSTRAINT external_completion_outbox_locator_check
            CHECK (btrim(locator) <> '');
    END IF;
END;
$$;

ALTER TABLE board.external_completion_outbox
    ADD COLUMN IF NOT EXISTS resolution_supported BOOLEAN NOT NULL DEFAULT false;
UPDATE board.external_completion_outbox
SET resolution_supported = true
WHERE resolved_at IS NOT NULL;
