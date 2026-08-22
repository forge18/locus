# tool-compaction — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `PreToolUse` hook binary in the base image | — | `cargo test -p locus-cli compact::hook_builds` |
| 2 | Materialize it into all eleven harnesses | 1 | `cargo test -p locus-core compact::materializes_everywhere` |
| 3 | Command rewrite table for verbose operations | 1 | `cargo test -p locus-core compact::rewrites` |
| 4 | Rewrite is visible in the event stream | 3 | `cargo test -p locus-core compact::rewrite_observable` |
| 5 | Result compaction before append | 1 | `cargo test -p locus-core compact::compacts_result` |
| 6 | Measure the saving as a ratio | 5 | `cargo test -p locus-core compact::saving_ratio` |
| 7 | Over-threshold results become `payload` artifacts | 5 | `cargo test -p locus-core compact::overflow_to_artifact` |
| 8 | Leave a one-line summary and an id in place | 7 | `cargo test -p locus-core compact::summary_with_handle` |
| 9 | Share one threshold setting with `artifacts` | 7 | `cargo test -p locus-core compact::single_threshold_setting` |
| 10 | Assert the hook never calls a model | 1 | `cargo test -p locus-cli compact::never_calls_a_model` |
| 11 | Assert the hook never blocks on the socket | 1 | `cargo test -p locus-cli compact::never_blocks` |
| 12 | Exit 0 on every failure path | 1 | `cargo test -p locus-cli compact::always_exits_zero` |
| 13 | A broken compactor degrades to no compaction | 12 | `cargo test -p locus-core compact::degrades_not_fails` |
| 14 | Offender ranking as a `GROUP BY` over `tool_result` rows | — | `cargo test -p locus-core compact::offender_ranking` |
| 15 | Rank per agent, per project, per harness | 14 | `cargo test -p locus-core compact::ranking_dimensions` |
| 16 | Compaction off changes cost but not behavior | 5 | `cargo test -p locus-core compact::behavior_unchanged` |
