# screens-wiki — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Three-pane frame at 246 / flex / 284 | — | `pnpm -C apps/desktop test -- wiki/layout` |
| 2 | Primary "Ingest a document" with the derived-then-curated note | 1 | `pnpm -C apps/desktop test -- wiki/ingest-primary` |
| 3 | Assert no blank-page action is more prominent than ingest | 2 | `pnpm -C apps/desktop test -- wiki/ingest-is-the-entry-point` |
| 4 | Typed tree groups with per-kind icons and counts | 1 | `pnpm -C apps/desktop test -- wiki/typed-groups` |
| 5 | Orphan flag in `--bad` on the offending entity | 4 | `pnpm -C apps/desktop test -- wiki/orphan-flag` |
| 6 | Selected page on `--sf2` with the accent ring | 4 | `pnpm -C apps/desktop test -- wiki/selected-page` |
| 7 | Article header: accent kind tag and 15px title | 1 | `pnpm -C apps/desktop test -- wiki/article-header` |
| 8 | Metadata row: mono locator, rev, assertion and source counts, ages | 7 | `pnpm -C apps/desktop test -- wiki/article-meta` |
| 9 | Prose at 13px/1.68, 88% opacity, max 720px, mono inline paths | 1 | `pnpm -C apps/desktop test -- wiki/prose` |
| 10 | `LINKS OUT` wikilink pills | 9 | `pnpm -C apps/desktop test -- wiki/links-out` |
| 11 | Wikilink pills navigate by locator | 10 | `pnpm -C apps/desktop test -- wiki/wikilink-navigates` |
| 12 | `PROVENANCE` list with icons | 1 | `pnpm -C apps/desktop test -- wiki/provenance` |
| 13 | `<WikiGraph>` 258x132 SVG built on the shared canvas renderer | 1 | `pnpm -C apps/desktop test -- wiki/graph` |
| 14 | Assert the graph imports the canvas renderer rather than duplicating it | 13 | `pnpm -C apps/desktop test -- wiki/graph-shares-renderer` |
| 15 | `CONTRADICTIONS` card with two values, two sources, two actions | 1 | `pnpm -C apps/desktop test -- wiki/contradiction-card` |
| 16 | Assert a contradiction always carries both sources | 15 | `pnpm -C apps/desktop test -- wiki/contradiction-is-adjudicable` |
| 17 | `LOCUS WIKI LINT` card with the four categories and the `--ok` clean line | 1 | `pnpm -C apps/desktop test -- wiki/lint-card` |
| 18 | Orphan count in the lint card matches the tree flags — one source | 5,17 | `pnpm -C apps/desktop test -- wiki/orphan-single-source` |
| 19 | Footer distinguishing the wiki from memory, verbatim | 1 | `pnpm -C apps/desktop test -- wiki/memory-distinction-note` |
| 20 | Visual check against `screenshots/14-wiki.png` | 18,19 | `pnpm -C apps/desktop test -- visual -- wiki` |
