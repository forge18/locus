# materializers — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `materialize/` module skeleton, strategy trait, and the extension-set input type | — | `cargo test -p locus-core materialize::trait_shape` |
| 2 | `dir` — copy as-is, with `suffix` and `flat` | 1 | `cargo test -p locus-core materialize::dir` |
| 3 | `merged-into` — render to one target, optional frontmatter strip | 1 | `cargo test -p locus-core materialize::merged_into` |
| 4 | `listed-in` — write paths into a config key | 1 | `cargo test -p locus-core materialize::listed_in` |
| 5 | `entries-in` — one structured entry per file | 1 | `cargo test -p locus-core materialize::entries_in` |
| 6 | `core-driven` — fire session_start/session_end from the container's lifetime | 1 | `cargo test -p locus-core materialize::core_driven` |
| 7 | Plugin host: JSON-RPC 2.0 over stdio, one call per run | 1 | `cargo test -p locus-core materialize::plugin_roundtrip` |
| 8 | Path containment check on every returned file | 7 | `cargo test -p locus-core materialize::plugin_path_escape_rejected` |
| 9 | Assert core writes the files; a plugin writing directly is refused | 7 | `cargo test -p locus-core materialize::plugin_returns_never_writes` |
| 10 | Deterministic file ordering across the whole tree | 2,3,4,5 | `cargo test -p locus-core materialize::sorted_file_order` |
| 11 | Deterministic list ordering inside generated files | 3,4,5 | `cargo test -p locus-core materialize::sorted_inner_lists` |
| 12 | No timestamps, run ids, or hostnames anywhere in output | 10,11 | `cargo test -p locus-core materialize::no_volatile_content` |
| 13 | Byte-identical trees across two runs of the same agent | 12 | `cargo test -p locus-core materialize::byte_identical_twice` |
| 14 | A deliberately non-deterministic materializer fails the determinism test | 13 | `cargo test -p locus-core materialize::detects_nondeterminism` |
| 15 | Config tree frozen for the run; a mid-run write fails | 13 | `cargo test -p locus-core materialize::tree_is_frozen` |
| 16 | pi's plugin: generate the TypeScript extension pi reads | 7 | `cargo test -p locus-core materialize::pi_plugin_generates` |
| 17 | pi actually loads the generated extension in a container | 16 | `cargo test -p locus-core materialize::pi_loads_generated -- --ignored` |
| 18 | Materialize all eight extensions for all twelve harnesses | 2,3,4,5,6,7 | `cargo test -p locus-core materialize::all_twelve_all_eight` |
| 19 | Materialization report carrying every `weaker_than_native` string | 18 | `cargo test -p locus-core materialize::report_carries_losses` |
| 20 | Expose the report over IPC for the Extensions screen | 19 | `pnpm -C apps/desktop test -- extensions/report-from-core` |
