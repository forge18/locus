# v2-knowledge-review — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add global Inbox v2 route and fixtures | — | `pnpm -C apps/desktop test -- inbox/v2-route` |
| 2 | Render action-required and completed inbox groups | 1 | `pnpm -C apps/desktop test -- inbox/v2-groups` |
| 3 | Render evidence, why-this-is-here, and cost-of-waiting detail | 2 | `pnpm -C apps/desktop test -- inbox/evidence-detail` |
| 4 | Render gate approve/send-back actions | 3 | `pnpm -C apps/desktop test -- inbox/gate-actions` |
| 5 | Add global Dashboard v2 route and fixtures | — | `pnpm -C apps/desktop test -- dashboard/v2-route` |
| 6 | Render project/running/model aggregate cards | 5 | `pnpm -C apps/desktop test -- dashboard/aggregates` |
| 7 | Render data-ramp charts without accent bars | 6 | `pnpm -C apps/desktop test -- dashboard/data-ramp` |
| 8 | Render steer-versus-review and review-debt measures | 6 | `pnpm -C apps/desktop test -- dashboard/trust-metrics` |
| 9 | Add selected-project Develop route | — | `pnpm -C apps/desktop test -- develop/v2-route` |
| 10 | Render project file tree and changed-file state | 9 | `pnpm -C apps/desktop test -- develop/file-tree` |
| 11 | Render split/unified diff controls and side-by-side measure | 10 | `pnpm -C apps/desktop test -- develop/diff-modes` |
| 12 | Render terminal session and linked-repo explanation | 9 | `pnpm -C apps/desktop test -- develop/terminal` |
| 13 | Render merge, PR, and chunk-revert actions | 11 | `pnpm -C apps/desktop test -- develop/review-actions` |
| 14 | Add selected-project Review telemetry route | — | `pnpm -C apps/desktop test -- review/telemetry-route` |
| 15 | Render telemetry filters, facets, and tool-error evidence | 14 | `pnpm -C apps/desktop test -- review/telemetry-filters` |
| 16 | Render run rows with model, tokens, cache, spend, and outcome | 14 | `pnpm -C apps/desktop test -- review/runs-table` |
| 17 | Resolve artifact links identically from review, memory, and inbox | 16 | `pnpm -C apps/desktop test -- artifact/one-viewer` |
| 18 | Add Memory Short-term route and context fixture | — | `pnpm -C apps/desktop test -- memory/short-term-route` |
| 19 | Render resident prompt layers and cache state | 18 | `pnpm -C apps/desktop test -- memory/resident-context` |
| 20 | Render compacted output with artifact handles | 19 | `pnpm -C apps/desktop test -- memory/compaction` |
| 21 | Render context ceiling and compaction threshold | 19 | `pnpm -C apps/desktop test -- memory/context-budget` |
| 22 | Add Memory Long-term route and fact fixture | — | `pnpm -C apps/desktop test -- memory/long-term-route` |
| 23 | Render provenance, confidence, decay, and contradiction state | 22 | `pnpm -C apps/desktop test -- memory/fact-provenance` |
| 24 | Add Memory Artifacts route and grouped artifact fixture | — | `pnpm -C apps/desktop test -- memory/artifacts-route` |
| 25 | Render artifact preview, comments, and review state | 24 | `pnpm -C apps/desktop test -- memory/artifact-preview` |
| 26 | Add Memory Wiki route and typed-page fixture | — | `pnpm -C apps/desktop test -- memory/wiki-route` |
| 27 | Render wiki outline, links, provenance, and graph | 26 | `pnpm -C apps/desktop test -- memory/wiki-viewer` |
| 28 | Add loading, empty, and error fixtures for each viewer | 1,5,9,14,18,22,24,26 | `pnpm -C apps/desktop test -- viewers/state-families` |
| 29 | Add locator, keyboard, and live-region coverage for each viewer | 28 | `pnpm -C apps/desktop test -- viewers/a11y` |
| 30 | Capture Dark/Light visual regressions for all viewer routes | 28 | `pnpm -C apps/desktop test -- visual/v2-viewers` |
