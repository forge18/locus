# wiki — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `wiki` schema: pages typed by kind, revisions, links, contradictions, ingest log | — | `cargo test -p locus-core wiki::schema` |
| 2 | The six page kinds as a closed enum | 1 | `cargo test -p locus-core wiki::six_kinds` |
| 3 | `markitdown` bridge for PDF, DOCX, PPTX, XLSX, HTML | — | `cargo test -p locus-core wiki::markitdown` |
| 4 | `locus wiki ingest <path\|url>` creating a `source` page | 3,1 | `cargo test -p locus-cli wiki::ingest` |
| 5 | Entity and concept extraction, auto-creating pages on first mention | 4 | `cargo test -p locus-core wiki::auto_pages` |
| 6 | Link pages by `[[wikilink]]` | 5 | `cargo test -p locus-core wiki::links` |
| 7 | `overview` revised on every ingest | 4 | `cargo test -p locus-core wiki::overview_revises` |
| 8 | Embed assertions into `pgvector` | 1 | `cargo test -p locus-core wiki::embeds` |
| 9 | k-nearest retrieval of existing assertions at ingest | 8 | `cargo test -p locus-core wiki::knn_at_ingest` |
| 10 | Adjudicate only the k nearest with a model | 9 | `cargo test -p locus-core wiki::bounded_adjudication` |
| 11 | Assert ingest cost scales with the document, not the wiki | 10 | `cargo test -p locus-core wiki::cost_is_bounded` |
| 12 | Write a contradiction row carrying both statements and both sources | 10 | `cargo test -p locus-core wiki::contradiction_row` |
| 13 | Raise a board card for each contradiction | 12 | `cargo test -p locus-core wiki::contradiction_card` |
| 14 | The same detection over a conflicting memory-store fact | 12 | `cargo test -p locus-core wiki::memory_conflict` |
| 15 | `locus wiki lint`: orphans, broken links, unnamed entities, unsourced assertions | 6 | `cargo test -p locus-cli wiki::lint` |
| 16 | `locus wiki search\|read\|write\|history` | 1 | `cargo test -p locus-cli wiki::verbs` |
| 17 | `locus wiki query` filing its answer as a `synthesis` page | 8 | `cargo test -p locus-cli wiki::query_files_synthesis` |
| 18 | Revisions attributed to the run that made them | 1 | `cargo test -p locus-core wiki::revision_attribution` |
| 19 | GUI editor writing back, readable by an agent in a container | 16 | `cargo test -p locus-core wiki::gui_edit_readable` |
| 20 | Graph view importing the canvas renderer | 6 | `pnpm -C apps/desktop test -- wiki/graph-shares-renderer` |
| 21 | Wire the Wiki screen to real pages | 16 | `pnpm -C apps/desktop test -- wiki/from-core` |
| 22 | Seed by ingesting the repo's own ADRs, specs and READMEs | 4 | `cargo test -p locus-core wiki::seeds_from_git` |
