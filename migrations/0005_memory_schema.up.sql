CREATE SCHEMA memory;

CREATE TABLE memory.core (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    agent_def_id UUID NOT NULL REFERENCES agents.agent_defs (id) ON DELETE CASCADE,
    path TEXT NOT NULL CHECK (btrim(path) <> ''),
    summary TEXT NOT NULL CHECK (btrim(summary) <> ''),
    body TEXT NOT NULL,
    token_count INTEGER NOT NULL CHECK (token_count >= 0),
    source_memory_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, agent_def_id, path)
);

CREATE INDEX memory_core_project_agent_idx ON memory.core (project_id, agent_def_id);

CREATE TABLE memory.store (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    scope TEXT NOT NULL CHECK (scope IN ('project', 'agent')),
    agent_def_id UUID REFERENCES agents.agent_defs (id) ON DELETE CASCADE,
    path TEXT NOT NULL CHECK (btrim(path) <> ''),
    subject TEXT NOT NULL CHECK (btrim(subject) <> ''),
    category TEXT NOT NULL CHECK (category IN ('strategy', 'fact', 'assumption', 'failure')),
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    provenance JSONB NOT NULL,
    source_run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    source_event_id UUID REFERENCES agents.events (id) ON DELETE SET NULL,
    embedding vector NOT NULL,
    embedding_model TEXT NOT NULL CHECK (btrim(embedding_model) <> ''),
    confidence DOUBLE PRECISION NOT NULL CHECK (confidence BETWEEN 0 AND 1),
    importance DOUBLE PRECISION NOT NULL CHECK (importance BETWEEN 0 AND 1),
    recall_count INTEGER NOT NULL DEFAULT 0 CHECK (recall_count >= 0),
    last_recalled_at TIMESTAMPTZ,
    active_days INTEGER NOT NULL DEFAULT 0 CHECK (active_days >= 0),
    strength DOUBLE PRECISION NOT NULL CHECK (strength BETWEEN 0 AND 1),
    last_strength_at TIMESTAMPTZ,
    keeper_match_checked_at TIMESTAMPTZ,
    invalidated_at TIMESTAMPTZ,
    reverify_requested_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    supersedes_id UUID REFERENCES memory.store (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (scope = 'project' AND agent_def_id IS NULL)
        OR (scope = 'agent' AND agent_def_id IS NOT NULL)
    ),
    CHECK (supersedes_id IS NULL OR supersedes_id <> id)
);

COMMENT ON COLUMN memory.store.importance IS
    'No default: the initial importance policy is intentionally undecided.';
COMMENT ON COLUMN memory.store.embedding IS
    'carve_out: model output, not reproducible from folded events; rebuild preserves bytes.';
COMMENT ON COLUMN memory.store.active_days IS
    'carve_out: wall-clock decay state evaluated at read, never emitted as a tick event.';
COMMENT ON COLUMN memory.store.strength IS
    'carve_out: derived decay state evaluated at read from last_active and the category curve.';

ALTER TABLE memory.core
    ADD CONSTRAINT memory_core_source_memory_id_fkey
    FOREIGN KEY (source_memory_id) REFERENCES memory.store (id) ON DELETE SET NULL;

CREATE INDEX memory_store_project_scope_idx
    ON memory.store (project_id, scope, agent_def_id);
CREATE INDEX memory_store_project_path_idx ON memory.store (project_id, path);
CREATE INDEX memory_store_project_category_idx
    ON memory.store (project_id, category) WHERE archived_at IS NULL AND invalidated_at IS NULL;
CREATE INDEX memory_store_supersedes_id_idx ON memory.store (supersedes_id);

CREATE TABLE memory.probation (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    agent_def_id UUID NOT NULL REFERENCES agents.agent_defs (id) ON DELETE CASCADE,
    path TEXT NOT NULL CHECK (btrim(path) <> ''),
    subject TEXT NOT NULL CHECK (btrim(subject) <> ''),
    category TEXT NOT NULL CHECK (category IN ('strategy', 'fact', 'assumption', 'failure')),
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    provenance JSONB NOT NULL,
    source_run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    source_event_id UUID REFERENCES agents.events (id) ON DELETE SET NULL,
    embedding vector NOT NULL,
    embedding_model TEXT NOT NULL CHECK (btrim(embedding_model) <> ''),
    confidence DOUBLE PRECISION NOT NULL CHECK (confidence BETWEEN 0 AND 1),
    score DOUBLE PRECISION NOT NULL CHECK (score >= 0),
    target_count INTEGER NOT NULL DEFAULT 1 CHECK (target_count > 0),
    promoted_at TIMESTAMPTZ,
    discarded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (promoted_at IS NULL OR discarded_at IS NULL)
);

CREATE INDEX memory_probation_project_path_idx
    ON memory.probation (project_id, path) WHERE promoted_at IS NULL AND discarded_at IS NULL;
CREATE INDEX memory_probation_expires_at_idx
    ON memory.probation (expires_at) WHERE promoted_at IS NULL AND discarded_at IS NULL;

CREATE TABLE memory.edges (
    source_memory_id UUID NOT NULL REFERENCES memory.store (id) ON DELETE CASCADE,
    target_memory_id UUID NOT NULL REFERENCES memory.store (id) ON DELETE CASCADE,
    relation TEXT NOT NULL CHECK (relation IN ('supports', 'contradicts', 'depends_on', 'derived_from')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (source_memory_id, target_memory_id, relation),
    CHECK (source_memory_id <> target_memory_id)
);

CREATE INDEX memory_edges_target_idx ON memory.edges (target_memory_id);
