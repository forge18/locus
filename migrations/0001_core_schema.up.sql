CREATE SCHEMA core;

CREATE TABLE core.projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.repos (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    working_copy_path TEXT NOT NULL CHECK (btrim(working_copy_path) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, working_copy_path)
);

CREATE INDEX repos_project_id_idx ON core.repos (project_id);

CREATE TABLE core.local_remotes (
    id UUID PRIMARY KEY,
    repo_id UUID NOT NULL UNIQUE REFERENCES core.repos (id) ON DELETE CASCADE,
    bare_path TEXT NOT NULL UNIQUE CHECK (btrim(bare_path) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE core.settings (
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    key TEXT NOT NULL CHECK (btrim(key) <> ''),
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, key)
);
