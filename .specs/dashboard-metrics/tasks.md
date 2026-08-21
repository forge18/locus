# dashboard-metrics — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Runs and spend aggregates | — | `cargo test -p locus-core metrics::runs_and_spend` |
| 2 | Cache rate from `usage.cache_read` over `usage.input` | 1 | `cargo test -p locus-core metrics::cache_rate` |
| 3 | Cache rate reads *unknown* where usage is null | 2 | `cargo test -p locus-core metrics::cache_unknown_not_zero` |
| 4 | Offender ranking by total result payload | — | `cargo test -p locus-core metrics::offender_ranking` |
| 5 | Slice the ranking by agent, project and harness | 4 | `cargo test -p locus-core metrics::ranking_slices` |
| 6 | Verify pass rate | — | `cargo test -p locus-core metrics::verify_pass_rate` |
| 7 | Guardrail trip counts by kind | — | `cargo test -p locus-core metrics::guardrail_trips` |
| 8 | Board throughput from transitions | — | `cargo test -p locus-core metrics::board_throughput` |
| 9 | Spec-gap and ambiguity-detection rates from the arbiter column | — | `cargo test -p locus-core metrics::arbiter_rates` |
| 10 | Average iterations per task | — | `cargo test -p locus-core metrics::iterations_per_task` |
| 11 | Review-gate precision | — | `cargo test -p locus-core metrics::gate_precision` |
| 12 | Agent trust over the last 20 runs | 6 | `cargo test -p locus-core metrics::agent_trust` |
| 13 | Discount trust by guardrail trips and rejected artifacts | 12 | `cargo test -p locus-core metrics::trust_discounts` |
| 14 | Discount trust by tokens per passing run | 12 | `cargo test -p locus-core metrics::trust_by_tokens` |
| 15 | Assert no metric added a new write path | 1,4,9,12 | `bash scripts/check-metrics-are-queries.sh` |
| 16 | Wire Status to the at-a-glance set | 2,6,7 | `pnpm -C apps/desktop test -- status/from-core` |
| 17 | Assert Status still has no query tool | 16 | `pnpm -C apps/desktop test -- status/no-query-tool` |
| 18 | Wire Review Telemetry and Runs to the full set with facets | 4,9,12 | `pnpm -C apps/desktop test -- telemetry/from-core` |
| 19 | Cache-rate alert threshold, decided and implemented | 2 | `cargo test -p locus-core metrics::cache_alert` |
| 20 | An unstable prefix drops cache rate and names the responsible run | 19 | `cargo test -p locus-core metrics::detects_prefix_drift` |
