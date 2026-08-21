CREATE TABLE core.model_tier_settings (
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    harness_name TEXT NOT NULL CHECK (btrim(harness_name) <> ''),
    tier TEXT NOT NULL CHECK (tier IN ('low', 'medium', 'high', 'xhigh')),
    model_id TEXT NOT NULL CHECK (btrim(model_id) <> ''),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, harness_name, tier)
);
