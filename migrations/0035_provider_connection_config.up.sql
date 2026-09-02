ALTER TABLE core.providers
    ADD COLUMN IF NOT EXISTS authentication_method TEXT NOT NULL DEFAULT 'api-key',
    ADD COLUMN IF NOT EXISTS base_url TEXT;

ALTER TABLE core.providers
    DROP CONSTRAINT IF EXISTS providers_authentication_method_check;

ALTER TABLE core.providers
    ADD CONSTRAINT providers_authentication_method_check
    CHECK (authentication_method IN ('oauth', 'api-key', 'none'));

ALTER TABLE core.providers
    ADD CONSTRAINT providers_base_url_check
    CHECK (base_url IS NULL OR btrim(base_url) <> '');
