DROP TABLE core.provider_models;

ALTER TABLE core.providers
    DROP CONSTRAINT providers_verification_metadata_check,
    DROP COLUMN verification_status,
    DROP COLUMN verification_model_count,
    DROP COLUMN verification_at;
