-- M1.6 records the machine-selected runtime on every agent run.
-- Existing rows remain Docker runs because Docker is the default backend.
ALTER TABLE agents.runs
    ADD COLUMN IF NOT EXISTS runtime_backend TEXT NOT NULL DEFAULT 'docker'
        CHECK (runtime_backend IN ('docker', 'sbx'));
