CREATE TABLE agents.credential_proxy_audits (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES agents.runs (id) ON DELETE CASCADE,
    target TEXT NOT NULL CHECK (target IN ('model', 'package', 'other')),
    tier TEXT NOT NULL CHECK (tier IN ('none', 'model', 'packages', 'open')),
    allowed BOOLEAN NOT NULL,
    credential_class TEXT NOT NULL CHECK (btrim(credential_class) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX credential_proxy_audits_run_id_created_at_idx
ON agents.credential_proxy_audits (run_id, created_at);
