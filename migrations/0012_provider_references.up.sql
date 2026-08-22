CREATE TABLE core.providers (
    id UUID PRIMARY KEY,
    identifier TEXT NOT NULL UNIQUE CHECK (btrim(identifier) <> ''),
    keychain_reference TEXT NOT NULL UNIQUE CHECK (
        btrim(keychain_reference) <> ''
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
