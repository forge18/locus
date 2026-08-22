CREATE SCHEMA agents;

CREATE TABLE agents.agent_defs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    version INTEGER NOT NULL CHECK (version > 0),
    frontmatter JSONB NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (name, version)
);

CREATE TABLE agents.sessions (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    agent_def_id UUID NOT NULL REFERENCES agents.agent_defs (id),
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    branch TEXT NOT NULL CHECK (btrim(branch) <> ''),
    board_task_id UUID,
    memory_base JSONB NOT NULL DEFAULT '{}'::jsonb,
    pane_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL CHECK (status IN ('active', 'closed')) DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ
);

CREATE INDEX sessions_project_id_idx ON agents.sessions (project_id);
CREATE INDEX sessions_agent_def_id_idx ON agents.sessions (agent_def_id);

CREATE TABLE agents.runs (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES agents.sessions (id) ON DELETE CASCADE,
    resolved_model_id TEXT NOT NULL CHECK (btrim(resolved_model_id) <> ''),
    container_id TEXT UNIQUE,
    harness_session_id TEXT,
    allocated_port INTEGER CHECK (allocated_port BETWEEN 43000 AND 43999),
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'paused', 'completed', 'aborted', 'cancelled')),
    usage JSONB,
    exit_code INTEGER,
    cancel_reason TEXT,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX runs_session_id_idx ON agents.runs (session_id);
CREATE INDEX runs_status_idx ON agents.runs (status);

CREATE TABLE agents.run_edges (
    parent_run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    child_run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL CHECK (edge_type IN ('spawned', 'resumed', 'handed_off')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (parent_run_id, child_run_id),
    CHECK (parent_run_id <> child_run_id)
);

CREATE TABLE agents.events (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    seq BIGINT NOT NULL CHECK (seq >= 0),
    ts TIMESTAMPTZ NOT NULL,
    verb TEXT NOT NULL CHECK (verb IN (
        'session_start', 'user', 'assistant', 'thinking', 'tool_call', 'tool_result',
        'tool_error', 'permission_request', 'subagent_start', 'subagent_stop', 'aborted',
        'session_end'
    )),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw JSONB NOT NULL,
    UNIQUE (run_id, seq)
);

CREATE INDEX events_run_id_seq_idx ON agents.events (run_id, seq);

CREATE TABLE agents.artifacts (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN (
        'plan', 'diff', 'diagram', 'image', 'recording', 'walkthrough', 'finding', 'payload'
    )),
    title TEXT,
    body TEXT,
    blob_path TEXT,
    media_type TEXT,
    sha256 TEXT,
    derived_representation JSONB,
    summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (body IS NOT NULL OR blob_path IS NOT NULL),
    CHECK (blob_path IS NULL OR (media_type IS NOT NULL AND sha256 IS NOT NULL))
);

CREATE INDEX artifacts_run_id_idx ON agents.artifacts (run_id);

CREATE TABLE agents.artifact_comments (
    id UUID PRIMARY KEY,
    artifact_id UUID NOT NULL REFERENCES agents.artifacts (id) ON DELETE CASCADE,
    parent_comment_id UUID REFERENCES agents.artifact_comments (id) ON DELETE CASCADE,
    author_kind TEXT NOT NULL CHECK (author_kind IN ('human', 'agent')),
    author_id UUID,
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX artifact_comments_artifact_id_idx ON agents.artifact_comments (artifact_id);
