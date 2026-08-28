CREATE TABLE board.external_work_item_providers (
    plugin_id TEXT PRIMARY KEY CHECK (btrim(plugin_id) <> ''),
    host TEXT NOT NULL CHECK (btrim(host) <> ''),
    provider_project TEXT NOT NULL CHECK (btrim(provider_project) <> ''),
    configured_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE board.external_work_items (
    task_id UUID PRIMARY KEY REFERENCES board.tasks (id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL CHECK (btrim(plugin_id) <> ''),
    host TEXT NOT NULL CHECK (btrim(host) <> ''),
    provider_project TEXT NOT NULL CHECK (btrim(provider_project) <> ''),
    native_id TEXT NOT NULL CHECK (btrim(native_id) <> ''),
    url TEXT NOT NULL CHECK (btrim(url) <> ''),
    title TEXT NOT NULL CHECK (btrim(title) <> ''),
    body TEXT NOT NULL DEFAULT '',
    labels JSONB NOT NULL DEFAULT '[]'::JSONB,
    source_status TEXT NOT NULL DEFAULT 'open',
    imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (plugin_id, host, provider_project, native_id)
);

CREATE TABLE board.external_completion_outbox (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL UNIQUE REFERENCES board.tasks (id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL CHECK (btrim(plugin_id) <> ''),
    host TEXT NOT NULL CHECK (btrim(host) <> ''),
    provider_project TEXT NOT NULL CHECK (btrim(provider_project) <> ''),
    native_id TEXT NOT NULL CHECK (btrim(native_id) <> ''),
    comment TEXT NOT NULL CHECK (btrim(comment) <> ''),
    evidence JSONB NOT NULL DEFAULT '[]'::JSONB,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    commented_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
