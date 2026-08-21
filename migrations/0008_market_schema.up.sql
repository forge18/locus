-- The marketplace index remains external. These rows are resolver snapshots and image-build
-- records, never the mutable source of truth for manifests or any M8 trust policy.
CREATE SCHEMA market;

CREATE TABLE market.manifest_snapshots (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    manifest JSONB NOT NULL,
    content_sha256 TEXT NOT NULL CHECK (btrim(content_sha256) <> ''),
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (name, content_sha256),
    UNIQUE (id, name)
);

CREATE INDEX market_manifest_snapshots_name_idx
    ON market.manifest_snapshots (name);

CREATE TABLE market.tool_sets (
    id UUID PRIMARY KEY,
    base_image_digest TEXT NOT NULL CHECK (btrim(base_image_digest) <> ''),
    image_cache_key TEXT NOT NULL UNIQUE CHECK (btrim(image_cache_key) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE market.tool_set_manifest_pins (
    tool_set_id UUID NOT NULL REFERENCES market.tool_sets (id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL CHECK (btrim(tool_name) <> ''),
    manifest_snapshot_id UUID NOT NULL,
    PRIMARY KEY (tool_set_id, tool_name),
    UNIQUE (tool_set_id, manifest_snapshot_id),
    FOREIGN KEY (manifest_snapshot_id, tool_name)
        REFERENCES market.manifest_snapshots (id, name) ON DELETE RESTRICT
);

CREATE INDEX market_tool_set_manifest_pins_snapshot_idx
    ON market.tool_set_manifest_pins (manifest_snapshot_id);

CREATE TABLE market.installs (
    id UUID PRIMARY KEY,
    tool_set_id UUID NOT NULL,
    manifest_snapshot_id UUID NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'verified', 'failed')),
    started_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (verified_at IS NULL OR verified_at >= created_at),
    UNIQUE (tool_set_id, manifest_snapshot_id),
    FOREIGN KEY (tool_set_id, manifest_snapshot_id)
        REFERENCES market.tool_set_manifest_pins (tool_set_id, manifest_snapshot_id)
        ON DELETE CASCADE
);

CREATE INDEX market_installs_tool_set_id_idx
    ON market.installs (tool_set_id);

CREATE TABLE market.agent_tool_set_resolutions (
    agent_def_id UUID PRIMARY KEY REFERENCES agents.agent_defs (id) ON DELETE CASCADE,
    tool_set_id UUID NOT NULL REFERENCES market.tool_sets (id) ON DELETE RESTRICT,
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX market_agent_tool_set_resolutions_tool_set_id_idx
    ON market.agent_tool_set_resolutions (tool_set_id);
