CREATE TABLE core.extensions (
    id UUID PRIMARY KEY,
    extension_type TEXT NOT NULL CHECK (
        extension_type IN (
            'agents',
            'skills',
            'rules',
            'context',
            'commands',
            'hooks',
            'output-styles',
            'linters'
        )
    ),
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    frontmatter JSONB NOT NULL DEFAULT '{}'::JSONB CHECK (
        jsonb_typeof(frontmatter) = 'object'
    ),
    body TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (extension_type, name)
);

CREATE TABLE core.extension_revisions (
    id UUID PRIMARY KEY,
    extension_id UUID NOT NULL REFERENCES core.extensions (
        id
    ) ON DELETE CASCADE,
    version INTEGER NOT NULL CHECK (version > 0),
    frontmatter JSONB NOT NULL CHECK (jsonb_typeof(frontmatter) = 'object'),
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (extension_id, version)
);
