# security — tasks

Forward-looking: F1, F2, F4 implement controls that do not exist yet, so their `verify:` commands name
tests to be written as part of the task. F3 is a code change to existing code and reuses the current
`provider::` tests.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Route raw provider error to the host trusted log; forward a secret-free gist upstream | — | `cargo test -p locus-core provider::` (extend with a "transformed echo is absent from Err" case) |
| 2 | Keep exact-match `redact()` as defense-in-depth; pin behaviour with a regression | 1 | `cargo test -p locus-core provider::` |
| 3 | Add per-run tier destination allowlists to the existing project-network proxy | — | `cargo test -p locus-core egress::tiers` (new) |
| 4 | Route all run egress through the forwarding proxy; apply tier filters | 3 | `cargo test -p locus-core egress::forwarded` (new) |
| 5 | Assert `None`-tier runs share no general network with egress-capable runs | 3 | `cargo test -p locus-core net::project_isolation` (extend) |
| 6 | Define the three trusted channels and the data-plane default in the materialized context | — | `cargo test -p locus-core materialize::trust_boundary` (new) |
| 7 | Emit the standing "data, never instructions" rule in always-on context | 6 | `cargo test -p locus-core materialize::standing_rule` (new) |
| 8 | Implement the override ladder: once / session / global, one interaction, non-blocking | 6 | `cargo test -p locus-core materialize::override_ladder` (new) |
| 9 | Guard the read-only config mount as a regression (no tool data renders into policy tree) | 6 | `cargo test -p locus-core container::host_tree_unreachable` (extend) |
| 10 | Add ownership check at the daemon socket for ID-bearing verbs, before routing | — | `cargo test -p locus-core agent_socket::` (extend) |
| 11 | Regression: a request scoped to run A cannot read run B's artifacts or settings | 10 | `cargo test -p locus-core agent_socket::cross_run_refused` (new) |
