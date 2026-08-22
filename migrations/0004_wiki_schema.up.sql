CREATE EXTENSION IF NOT EXISTS vector;

CREATE SCHEMA wiki;

CREATE TABLE wiki.pages (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN (
        'source', 'entity', 'concept', 'synthesis', 'decision', 'overview'
    )),
    slug TEXT NOT NULL CHECK (btrim(slug) <> ''),
    title TEXT NOT NULL CHECK (btrim(title) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, kind, slug)
);

CREATE INDEX pages_project_id_idx ON wiki.pages (project_id);
CREATE INDEX pages_project_kind_idx ON wiki.pages (project_id, kind);

CREATE TABLE wiki.revisions (
    id UUID PRIMARY KEY,
    page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    body TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    author_kind TEXT NOT NULL CHECK (author_kind IN ('human', 'agent', 'system')),
    author_run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (page_id, revision_number),
    CHECK (author_kind <> 'agent' OR author_run_id IS NOT NULL)
);

CREATE INDEX revisions_page_id_created_at_idx ON wiki.revisions (page_id, created_at DESC);

CREATE FUNCTION wiki.reject_revision_update() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'wiki revisions are immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER revisions_are_immutable
BEFORE UPDATE ON wiki.revisions
FOR EACH ROW EXECUTE FUNCTION wiki.reject_revision_update();

CREATE TABLE wiki.links (
    source_page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    source_revision_id UUID NOT NULL REFERENCES wiki.revisions (id) ON DELETE CASCADE,
    target_page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (source_revision_id, target_page_id),
    CHECK (source_page_id <> target_page_id)
);

CREATE INDEX links_source_page_id_idx ON wiki.links (source_page_id);
CREATE INDEX links_target_page_id_idx ON wiki.links (target_page_id);

CREATE TABLE wiki.ingest_log (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    source_page_id UUID REFERENCES wiki.pages (id) ON DELETE SET NULL,
    input_kind TEXT NOT NULL CHECK (input_kind IN ('path', 'url')),
    input_locator TEXT NOT NULL CHECK (btrim(input_locator) <> ''),
    content_sha256 TEXT,
    status TEXT NOT NULL CHECK (status IN ('started', 'completed', 'failed')),
    run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    result JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CHECK ((status = 'started' AND completed_at IS NULL) OR (status <> 'started' AND completed_at IS NOT NULL)),
    CHECK ((status = 'failed') = (error IS NOT NULL))
);

CREATE INDEX ingest_log_project_started_at_idx ON wiki.ingest_log (project_id, started_at DESC);
CREATE INDEX ingest_log_source_page_id_idx ON wiki.ingest_log (source_page_id);

CREATE TABLE wiki.embeddings (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    revision_id UUID NOT NULL REFERENCES wiki.revisions (id) ON DELETE CASCADE,
    source_page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    statement TEXT NOT NULL CHECK (btrim(statement) <> ''),
    embedding vector NOT NULL,
    model TEXT NOT NULL CHECK (btrim(model) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX embeddings_project_model_idx ON wiki.embeddings (project_id, model);
CREATE INDEX embeddings_revision_id_idx ON wiki.embeddings (revision_id);
CREATE INDEX embeddings_source_page_id_idx ON wiki.embeddings (source_page_id);

CREATE TABLE wiki.contradictions (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    existing_embedding_id UUID NOT NULL REFERENCES wiki.embeddings (id) ON DELETE CASCADE,
    new_embedding_id UUID NOT NULL REFERENCES wiki.embeddings (id) ON DELETE CASCADE,
    existing_statement TEXT NOT NULL CHECK (btrim(existing_statement) <> ''),
    new_statement TEXT NOT NULL CHECK (btrim(new_statement) <> ''),
    existing_source_page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    new_source_page_id UUID NOT NULL REFERENCES wiki.pages (id) ON DELETE CASCADE,
    adjudication JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL CHECK (status IN ('open', 'resolved', 'dismissed')) DEFAULT 'open',
    resolved_by_run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (existing_embedding_id <> new_embedding_id),
    CHECK (existing_source_page_id <> new_source_page_id),
    CHECK ((status = 'open' AND resolved_at IS NULL) OR (status <> 'open' AND resolved_at IS NOT NULL))
);

CREATE INDEX contradictions_project_status_idx ON wiki.contradictions (project_id, status);
CREATE INDEX contradictions_existing_embedding_id_idx ON wiki.contradictions (existing_embedding_id);
CREATE INDEX contradictions_new_embedding_id_idx ON wiki.contradictions (new_embedding_id);
