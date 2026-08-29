# context-layer verification

The complete 14-task batch was implemented and verified in dependency order.

## Task checks

- `cargo test -p locus-core memory::eviction_class_schema --no-fail-fast`
- `cargo test -p locus-core memory::eviction_class_assignment --no-fail-fast`
- `cargo test -p locus-core memory::catalog_overflow_respects_eviction_class --no-fail-fast`
- `cargo test -p locus-core materializer::context_derivation_table --no-fail-fast`
- `cargo test -p locus-core materializer::derivation_keeps_constitution_head --no-fail-fast`
- `cargo test -p locus-cli memory::promote_verb --no-fail-fast`
- `cargo test -p locus-core memory::promoted_decay_normally --no-fail-fast`
- `cargo test -p locus-core memory::research_diversity_dedup --no-fail-fast`
- `cargo test -p locus-core memory::k1_selection_unchanged --no-fail-fast`
- `cargo test -p locus-core memory::catalog_tail_append --no-fail-fast`
- `cargo test -p locus-core memory::snapshot_head_stable --no-fail-fast`
- `cargo test -p locus-core run_supervisor::recitation_block --no-fail-fast`
- `cargo test -p locus-core calibration::cache_rate_criterion --no-fail-fast`
- `cargo test -p locus-core telemetry::attribution_view --no-fail-fast`

All listed task checks passed.

## Project gates

- `cargo fmt --all -- --check` — passed.
- `cargo test` — passed: 911 passed, 3 ignored.
- `cargo clippy --all-targets -- -D warnings` — passed.
- `just ci` — passed, including desktop build, harness lint, deterministic materialization, ignored Docker smoke checks, and repository policy checks.
- `just typecheck` — passed.
- `just test-node` — passed: 356 files, 961 tests.
- `git diff --check` — passed.
- `lens_diagnostics` on edited source files — no issues.

The schema integration suite exercised the new `agents.context_attribution` view and
`memory.store.eviction_class` column through the embedded migrations.
