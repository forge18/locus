# Locus build steps. Each recipe wraps the command it replaces verbatim — the justfile
# names commands, it never changes them. See .specs/justfile/spec.md.

set positional-arguments

# Install Rust crates and desktop dependencies
setup:
    cargo fetch && pnpm -C apps/desktop install

# Build the Rust workspace and the desktop app
build:
    cargo build && pnpm -C apps/desktop build

# Run the Rust test suite after verifying Docker is reachable
test:
    bash scripts/check-docker.sh
    cargo test

# Run the desktop (Node) test suite
test-node:
    pnpm -C apps/desktop test

# Run one named test; exits non-zero when the filter matches nothing
test-named *args:
    bash scripts/run-named-test.sh "$@"

# Run the real Tauri window against a disposable Postgres store
test-desktop-integration:
    bash scripts/check-docker.sh
    bash apps/desktop/scripts/test-desktop-integration.sh

# Run the real Tauri window inside a Linux Docker container with Xvfb
test-desktop-integration-linux:
    bash scripts/check-docker.sh
    bash apps/desktop/scripts/test-desktop-integration-linux.sh

# Lint the Rust workspace; a warning is a failure
lint:
    cargo clippy --all-targets -- -D warnings

# Typecheck the desktop app
typecheck:
    pnpm -C apps/desktop typecheck

# Run the desktop app in development mode
dev:
    pnpm -C apps/desktop tauri dev

# Full CI sequence, mirroring .github/workflows/ci.yml step-for-step (locusd smoke stays CI-only)
ci: test lint
    just test-node
    just typecheck
    pnpm -C apps/desktop build
    just test-named locus-core all_registered_harnesses --ignored
    just test-named locus-core isolates_failure --ignored
    bash scripts/check-no-harness-names-in-core.sh
    bash scripts/check-layering.sh
    bash scripts/check-lint-not-in-hooks.sh
    bash scripts/check-no-silent-skips.sh
    bash apps/desktop/scripts/check-counts-follow-registry.sh
    bash apps/desktop/scripts/check-no-literal-counts.sh
