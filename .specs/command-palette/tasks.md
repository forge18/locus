# command-palette — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Palette component over Kobalte, opened by `Cmd-K` | — | `pnpm -C apps/desktop test -- palette/opens` |
| 2 | Resolve a pasted locator of every kind | 1 | `pnpm -C apps/desktop test -- palette/resolves-all-kinds` |
| 3 | Unresolvable locator names the bad segment | 2 | `pnpm -C apps/desktop test -- palette/bad-locator-message` |
| 4 | Assert the palette holds no navigation logic of its own | 2 | `pnpm -C apps/desktop test -- palette/delegates-to-resolver` |
| 5 | `Cmd-P` global search over code | 1 | `cargo test -p locus-core palette::search_code` |
| 6 | Global search over wiki pages | 5 | `cargo test -p locus-core palette::search_wiki` |
| 7 | Global search over board tasks | 5 | `cargo test -p locus-core palette::search_tasks` |
| 8 | Global search over run history | 5 | `cargo test -p locus-core palette::search_runs` |
| 9 | Every result is a locator | 5,6,7,8 | `cargo test -p locus-core palette::results_are_locators` |
| 10 | Interleave and rank the four kinds in one list | 9 | `cargo test -p locus-core palette::unified_ranking` |
| 11 | Cross-project by default, each result labeled | 10 | `cargo test -p locus-core palette::cross_project` |
| 12 | Reuse `project-search` for content rather than reimplementing | 5 | `cargo test -p locus-core palette::reuses_project_search` |
| 13 | Back and forward over a locator stack | 2 | `pnpm -C apps/desktop test -- palette/history` |
| 14 | Assert all seven entry points call the one resolver | 4 | `bash apps/desktop/scripts/check-single-resolver.sh` |
