ALTER TABLE core.providers
    DROP CONSTRAINT IF EXISTS providers_base_url_check,
    DROP CONSTRAINT IF EXISTS providers_authentication_method_check,
    DROP COLUMN IF EXISTS base_url,
    DROP COLUMN IF EXISTS authentication_method;
