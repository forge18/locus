CREATE SCHEMA mail;

CREATE TABLE mail.threads (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES core.projects (id) ON DELETE CASCADE,
    subject TEXT NOT NULL CHECK (btrim(subject) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX mail_threads_project_id_idx ON mail.threads (project_id);

CREATE TABLE mail.messages (
    id UUID PRIMARY KEY,
    thread_id UUID NOT NULL REFERENCES mail.threads (id) ON DELETE CASCADE,
    sender_kind TEXT NOT NULL CHECK (sender_kind IN ('agent', 'human', 'system')),
    sender_run_id UUID REFERENCES agents.runs (id) ON DELETE SET NULL,
    body TEXT NOT NULL CHECK (btrim(body) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX mail_messages_thread_id_idx ON mail.messages (thread_id, created_at);

CREATE TABLE mail.deliveries (
    id UUID PRIMARY KEY,
    message_id UUID NOT NULL REFERENCES mail.messages (id) ON DELETE CASCADE,
    recipient_kind TEXT NOT NULL CHECK (recipient_kind IN ('agent', 'human')),
    recipient_session_id UUID REFERENCES agents.sessions (id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'delivered', 'read', 'drained')) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (recipient_kind = 'agent' AND recipient_session_id IS NOT NULL)
        OR (recipient_kind = 'human' AND recipient_session_id IS NULL)
    ),
    UNIQUE (message_id, recipient_session_id)
);

CREATE UNIQUE INDEX mail_deliveries_human_recipient_key
    ON mail.deliveries (message_id)
    WHERE recipient_kind = 'human';
CREATE INDEX mail_deliveries_recipient_session_pending_idx
    ON mail.deliveries (recipient_session_id, created_at)
    WHERE status = 'pending';

CREATE TABLE mail.waits (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    reason TEXT NOT NULL CHECK (reason IN ('ask', 'mail', 'debug-paused', 'gate')),
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ,
    CHECK (ended_at IS NULL OR ended_at >= started_at)
);

CREATE UNIQUE INDEX mail_waits_active_run_key
    ON mail.waits (run_id)
    WHERE ended_at IS NULL;
