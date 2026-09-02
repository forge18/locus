CREATE TABLE core.cli_tools (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE CHECK (btrim(name) <> ''),
    version TEXT NOT NULL CHECK (btrim(version) <> ''),
    category TEXT NOT NULL CHECK (category IN ('source-control', 'search-files', 'rust', 'database', 'web-network')),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    source TEXT NOT NULL DEFAULT 'builtin' CHECK (source IN ('builtin', 'uploaded')),
    binary_sha256 TEXT CHECK (binary_sha256 IS NULL OR binary_sha256 ~ '^[0-9a-f]{64}$'),
    install_command TEXT NOT NULL,
    verify_command TEXT NOT NULL,
    documentation_url TEXT,
    last_rebuilt_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO core.cli_tools
    (id, name, version, category, install_command, verify_command, documentation_url)
VALUES
    ('00000000-0000-0000-0000-000000000101', 'git', '2.49', 'source-control', 'apt-get install git', 'git --version', 'https://git-scm.com/docs'),
    ('00000000-0000-0000-0000-000000000102', 'rg', '14.1', 'search-files', 'apt-get install ripgrep', 'rg --version', 'https://github.com/BurntSushi/ripgrep'),
    ('00000000-0000-0000-0000-000000000103', 'cargo', '1.97', 'rust', 'rustup component add rustfmt', 'cargo --version', 'https://doc.rust-lang.org/cargo/'),
    ('00000000-0000-0000-0000-000000000104', 'rustc', '1.97', 'rust', 'rustup toolchain install stable', 'rustc --version', 'https://doc.rust-lang.org/rustc/'),
    ('00000000-0000-0000-0000-000000000105', 'psql', '17', 'database', 'apt-get install postgresql-client', 'psql --version', 'https://www.postgresql.org/docs/'),
    ('00000000-0000-0000-0000-000000000106', 'docker', '29', 'database', 'install docker', 'docker version', 'https://docs.docker.com/'),
    ('00000000-0000-0000-0000-000000000107', 'curl', '8', 'web-network', 'apt-get install curl', 'curl --version', 'https://curl.se/docs/'),
    ('00000000-0000-0000-0000-000000000108', 'jq', '1.7', 'web-network', 'apt-get install jq', 'jq --version', 'https://jqlang.github.io/jq/')
ON CONFLICT (name) DO NOTHING;
