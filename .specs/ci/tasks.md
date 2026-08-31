# ci — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | GitHub Actions workflow: Rust toolchain, node, pnpm, docker | — | `act -n -W .github/workflows/ci.yml` |
| 2 | `cargo test` step | 1 | `cargo test` |
| 3 | `cargo clippy --all-targets -- -D warnings` | 1 | `cargo clippy --all-targets -- -D warnings` |
| 4 | `pnpm -C apps/desktop build` step | 1 | `pnpm -C apps/desktop build` |
| 5 | `locus harness lint` step | 1 | `cargo run -p locus-cli -- harness lint` |
| 6 | Canary skill and canary rule fixtures | — | `test -s tests/canary/skill.md -a -s tests/canary/rule.md` |
| 7 | Smoke test: start a run and assert the agent sees both canaries | 6 | `cargo test -p locus-core smoke::canary_visible -- --ignored` |
| 8 | Run the smoke test per registered harness, failing by name | 7 | `cargo test -p locus-core smoke::all_registered_harnesses -- --ignored` |
| 9 | Breaking one harness's `via` fails only that harness | 8 | `cargo test -p locus-core smoke::isolates_failure -- --ignored` |
| 10 | Registration gate: a harness is not registered until its smoke test passes | 8 | `cargo test -p locus-core registry::smoke_gates_registration` |
| 11 | Determinism check: materialize twice, assert `diff -r` empty | — | `cargo test -p locus-core materialize::ci_determinism` |
| 12 | Determinism check fails on an injected timestamp | 11 | `cargo test -p locus-core materialize::ci_detects_timestamp` |
| 13 | Event-assertion helpers usable from any integration test | — | `cargo test -p locus-core testkit::event_assertions` |
| 14 | Mark docker-dependent tests explicitly; never skip silently | 7,11 | `bash scripts/check-no-silent-skips.sh` |
| 15 | Wire all checks into the workflow with clear failure names | 2,3,4,5,8,11 | `act -n -W .github/workflows/ci.yml` |
| 16 | Desktop test suite runs through the root recipe | 1 | `just test-node` |
| 17 | Desktop typecheck runs through the root recipe | 1 | `just typecheck` |
| 18 | Registry-derived fixture freshness gates run in CI | 4,6 | `bash apps/desktop/scripts/check-counts-follow-registry.sh && bash apps/desktop/scripts/check-no-literal-counts.sh` |
