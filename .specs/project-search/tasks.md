# project-search — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Content search over one repo | — | `cargo test -p locus-core search::single_repo` |
| 2 | Fan out across every repo the project holds | 1 | `cargo test -p locus-core search::all_project_repos` |
| 3 | Label every result with its repo | 2 | `cargo test -p locus-core search::results_carry_repo` |
| 4 | Rank results from different repos in one list | 2 | `cargo test -p locus-core search::unified_ranking` |
| 5 | Respect the project scope filter | 2 | `cargo test -p locus-core search::respects_scope` |
| 6 | `codanna` symbol search where indexed | 2 | `cargo test -p locus-core search::symbols` |
| 7 | Degrade to content search where not indexed | 6 | `cargo test -p locus-core search::degrades_gracefully` |
| 8 | Assert search never reads an agent's run clone | 2 | `cargo test -p locus-core search::never_reads_run_clones` |
| 9 | Search UI in the Develop category | 2 | `pnpm -C apps/desktop test -- develop/search` |
| 10 | Opening a result opens the file at the matching line | 9 | `pnpm -C apps/desktop test -- develop/search-opens-at-line` |
| 11 | Decide and implement the `codanna` index trigger | 6 | `cargo test -p locus-core search::index_trigger` |
