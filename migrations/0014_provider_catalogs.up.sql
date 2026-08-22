ALTER TABLE core.providers
ADD COLUMN verification_at TIMESTAMPTZ,
ADD COLUMN verification_model_count INTEGER,
ADD COLUMN verification_status TEXT,
ADD constraint PROVIDERS_VERIFICATION_METADATA_CHECK CHECK (
    (
        verification_at IS NULL
        AND verification_model_count IS NULL
        AND verification_status IS NULL
    )
    OR
    (
        verification_at IS NOT NULL
        AND verification_model_count IS NOT NULL
        AND verification_model_count >= 0
        AND verification_status IS NOT NULL
        AND verification_status IN ('verified', 'failed')
    )
);

CREATE TABLE core.provider_models (
    provider_id UUID NOT NULL REFERENCES core.providers (id) ON DELETE CASCADE,
    model_id TEXT NOT NULL CHECK (btrim(model_id) <> ''),
    alias TEXT CHECK (alias IS NULL OR btrim(alias) <> ''),
    selector_included BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (provider_id, model_id)
);
