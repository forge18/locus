# screens-review — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `<TelemetryView>` scrolling frame | — | `pnpm -C apps/desktop test -- telemetry/layout` |
| 2 | Search bar with blinking caret and the BM25 note | 1 | `pnpm -C apps/desktop test -- telemetry/search` |
| 3 | Filter chips with a Reset control | 1 | `pnpm -C apps/desktop test -- telemetry/filter-chips` |
| 4 | Four metric cards; Tool errors in `--bad` with a red hairline | 1 | `pnpm -C apps/desktop test -- telemetry/metrics` |
| 5 | Sparkline card of 16 accent bars | 1 | `pnpm -C apps/desktop test -- telemetry/sparkline` |
| 6 | Three-column 434px band frame | 1 | `pnpm -C apps/desktop test -- telemetry/band` |
| 7 | `<FacetGroup>` chips on `--sf3` with counts | 6 | `pnpm -C apps/desktop test -- telemetry/facets` |
| 8 | Active facet: accent tint plus accent inset ring | 7 | `pnpm -C apps/desktop test -- telemetry/facet-active` |
| 9 | Branch facet renders `main 0` at reduced opacity | 7 | `pnpm -C apps/desktop test -- telemetry/branch-invariant` |
| 10 | `<ActionRows>` — 132px label, 7px track, right-aligned count | 6 | `pnpm -C apps/desktop test -- telemetry/actions` |
| 11 | Assert the action list is exactly the twelve canonical verbs | 10 | `pnpm -C apps/desktop test -- telemetry/twelve-verbs` |
| 12 | `permission_request` renders as an alarm callout, not a count row | 10 | `pnpm -C apps/desktop test -- telemetry/permission-alarm` |
| 13 | Missing-verb note rendered beside the action list | 10 | `pnpm -C apps/desktop test -- telemetry/missing-verb-note` |
| 14 | `<ToolRows>` at 112px labels with the anomaly note | 6 | `pnpm -C apps/desktop test -- telemetry/tools` |
| 15 | Sessions table with mono right-aligned numerics and colored status | 1 | `pnpm -C apps/desktop test -- telemetry/sessions-table` |
| 16 | `<RunsView>` with search, `.seg` range control, and counts | — | `pnpm -C apps/desktop test -- runs/header` |
| 17 | Three right-aligned stats: spec-gap rate, noise reclassified, tokens per passing run | 16 | `pnpm -C apps/desktop test -- runs/stats` |
| 18 | Runs table with a Model-resolved column | 16 | `pnpm -C apps/desktop test -- runs/table` |
| 19 | Assert Model resolved holds a model id, never a tier name | 18 | `pnpm -C apps/desktop test -- runs/resolved-not-tier` |
| 20 | `<ArtifactsView>` three-pane frame at 222 / flex / 306 | — | `pnpm -C apps/desktop test -- artifacts/layout` |
| 21 | Review-kind list, one entry per kind | 20 | `pnpm -C apps/desktop test -- artifacts/review-kinds` |
| 22 | Dimmed reference group labeled never-in-the-inbox | 20 | `pnpm -C apps/desktop test -- artifacts/reference-group` |
| 23 | Assert reference kinds never appear in the inbox fixture | 22 | `pnpm -C apps/desktop test -- artifacts/reference-not-in-inbox` |
| 24 | Artifact header: kind tag, mono file name, locator, the one-viewer note | 20 | `pnpm -C apps/desktop test -- artifacts/header` |
| 25 | `<UnifiedDiff>` with `@@` headers, 26px gutter, Develop tints | 20 | `pnpm -C apps/desktop test -- artifacts/unified-diff` |
| 26 | Commented line carries `inset 3px 0 0 var(--ac)` | 25 | `pnpm -C apps/desktop test -- artifacts/commented-line` |
| 27 | Comment thread rail: your comment, agent reply, pulsing live note | 20 | `pnpm -C apps/desktop test -- artifacts/comments` |
| 28 | Rail footer with Send-to-session and Resolve | 27 | `pnpm -C apps/desktop test -- artifacts/comment-actions` |
| 29 | Same artifact renders identically from all three entry points | 24 | `pnpm -C apps/desktop test -- artifacts/one-viewer-three-entries` |
| 30 | Visual check against `screenshots/07`, `08`, `09` | 15,19,29 | `pnpm -C apps/desktop test -- visual -- telemetry runs artifact` |
