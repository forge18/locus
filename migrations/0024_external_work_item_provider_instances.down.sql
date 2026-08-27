DO $$
BEGIN
    IF EXISTS (
        SELECT plugin_id
        FROM board.external_work_item_providers
        GROUP BY plugin_id
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'cannot downgrade provider instances while multiple repositories share a plugin';
    END IF;
END;
$$;

ALTER TABLE board.external_completion_outbox
    DROP CONSTRAINT IF EXISTS external_completion_outbox_locator_check;
ALTER TABLE board.external_completion_outbox
    DROP COLUMN IF EXISTS resolution_supported,
    DROP COLUMN IF EXISTS locator;

ALTER TABLE board.external_work_items
    DROP CONSTRAINT IF EXISTS external_work_items_workflow_def_id_fkey,
    DROP COLUMN IF EXISTS workflow_def_id;

ALTER TABLE board.external_work_item_providers
    DROP CONSTRAINT IF EXISTS external_work_item_providers_pkey;
ALTER TABLE board.external_work_item_providers
    ADD CONSTRAINT external_work_item_providers_pkey PRIMARY KEY (plugin_id);
