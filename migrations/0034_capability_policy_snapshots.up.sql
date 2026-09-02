CREATE TABLE core.project_capability_policies (
    project_id UUID PRIMARY KEY REFERENCES core.projects (id) ON DELETE CASCADE,
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    policies JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO core.project_capability_policies (project_id)
SELECT id FROM core.projects
ON CONFLICT (project_id) DO NOTHING;

ALTER TABLE agents.runs
    ADD COLUMN capability_policy_revision INTEGER,
    ADD COLUMN capability_snapshot JSONB;
