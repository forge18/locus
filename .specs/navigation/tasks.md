# navigation — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `src/nav/views.ts` — the view/category/label table as one exported constant | — | `pnpm -C apps/desktop test -- nav/view-table` |
| 2 | `src/nav/tabs.ts` — tab sets per category, Automate's in Kanban-then-List order | 1 | `pnpm -C apps/desktop test -- nav/tab-sets` |
| 3 | Locator grammar parser: `parse(locator)` over all six object kinds | — | `pnpm -C apps/desktop test -- nav/locator-parse` |
| 4 | `format(view, params) → locator`, the parser's inverse | 3 | `pnpm -C apps/desktop test -- nav/locator-format` |
| 5 | Round-trip property test across every view and kind | 3,4 | `pnpm -C apps/desktop test -- nav/locator-roundtrip` |
| 6 | Reject a malformed locator with a message naming the bad segment | 3 | `pnpm -C apps/desktop test -- nav/locator-rejects` |
| 7 | `resolve(locator) → {view, params}` as the single navigation entry point | 3,1 | `pnpm -C apps/desktop test -- nav/resolver` |
| 8 | Nav store: `view` signal plus derived category, label, locator, visible tabs | 1,2,7 | `pnpm -C apps/desktop test -- nav/store-derives` |
| 9 | Rail click → the category's documented first view | 8 | `pnpm -C apps/desktop test -- nav/m07-shell-revision` |
| 10 | Tab click → that view, instant, no transition | 8 | `pnpm -C apps/desktop test -- nav/tab-click` |
| 11 | `agents` as an Extensions drill-down: Extensions stays lit, back link renders | 8 | `pnpm -C apps/desktop test -- nav/tab-sets` |
| 12 | Assert no Agents tab exists in the Workshop tab set | 2,11 | `pnpm -C apps/desktop test -- nav/tab-sets` |
| 13 | Assert the current category list is closed at runtime | 1 | `pnpm -C apps/desktop test -- nav/m07-shell-revision` |
| 14 | `Cmd-K` opens the locator bar and resolves what is typed | 7 | `pnpm -C apps/desktop test -- nav/cmd-k-resolves` |
| 15 | `openDetail()` renders a Sheet over the current category, leaving the rail unchanged | 7 | `pnpm -C apps/desktop test -- nav/detail-in-place` |
| 16 | Per-window back/forward as a stack of locators | 7 | `pnpm -C apps/desktop test -- nav/history-stack` |
| 17 | Project filter as scope only — asserted not to change `view` | 8 | `pnpm -C apps/desktop test -- nav/project-is-filter` |
| 18 | Lint: no component may set `view` outside the nav store | 8 | `bash apps/desktop/scripts/check-single-resolver.sh` |
