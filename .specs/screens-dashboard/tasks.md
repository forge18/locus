# screens-dashboard — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `<InboxView>` two-pane frame, left 392px with a right hairline | — | `pnpm -C apps/desktop test -- inbox/layout` |
| 2 | `NEEDS YOU` header with count and the "silence is the default" note | 1 | `pnpm -C apps/desktop test -- inbox/needs-you-header` |
| 3 | `<InboxCard>` with kind icon, title, right-aligned age, and the project/agent/branch subline | 1 | `pnpm -C apps/desktop test -- inbox/card` |
| 4 | Selected card: `--sf2` + accent inset ring; the three kind variants (gate, ask, guardrail) | 3 | `pnpm -C apps/desktop test -- inbox/card-variants` |
| 5 | `RESOLVED TODAY` list at `opacity:.6` | 1 | `pnpm -C apps/desktop test -- inbox/resolved` |
| 6 | Detail pane header: accent kind tag, 15px title, mono metadata row | 1 | `pnpm -C apps/desktop test -- inbox/detail-header` |
| 7 | Plan-gate body: accent `PLAN` label, numbered steps, mono inline paths, `info` callout | 6 | `pnpm -C apps/desktop test -- inbox/plan-body` |
| 8 | Comment textarea with the "comment steers the agent that made it" caption | 6 | `pnpm -C apps/desktop test -- inbox/comment-box` |
| 9 | Footer bar: primary approve, secondary send-back, the right-hand note | 6 | `pnpm -C apps/desktop test -- inbox/footer` |
| 10 | Approve resolves in place — view and rail unchanged | 9 | `pnpm -C apps/desktop test -- inbox/resolves-in-place` |
| 11 | "Open the work" navigates by locator into Plan, Develop or Review | 6 | `pnpm -C apps/desktop test -- inbox/work-routes-out` |
| 12 | Empty state: "Nothing needs you", no spinner | 2 | `pnpm -C apps/desktop test -- inbox/empty-is-silent` |
| 13 | `<StatusView>` scrolling column at the documented padding and gaps | — | `pnpm -C apps/desktop test -- status/layout` |
| 14 | `<MetricCard>` — 27px numeral, 15px unit, 10px uppercase label | 13 | `pnpm -C apps/desktop test -- status/metric-card` |
| 15 | Six-card grid with "Waiting on me" as the accent variant | 14 | `pnpm -C apps/desktop test -- status/six-metrics` |
| 16 | Cache read renders *unknown* rather than 0% when usage is null | 14 | `pnpm -C apps/desktop test -- status/unknown-not-zero` |
| 17 | `<RunsByHour>` — 12 stacked bars, 118px, three states stacked bottom-up | 13 | `pnpm -C apps/desktop test -- status/runs-by-hour` |
| 18 | `<WantsAttention>` — stuck, idle, and waiting rows with their icons | 13 | `pnpm -C apps/desktop test -- status/wants-attention` |
| 19 | The waiting row reads "waiting: gate — not idle" and differs from idle | 18 | `pnpm -C apps/desktop test -- status/waiting-not-idle` |
| 20 | Project table with `--ok`/`--bad` verify coloring and mono numerics | 13 | `pnpm -C apps/desktop test -- status/project-table` |
| 21 | Assert Status has no search, filter chips, or facets | 13 | `pnpm -C apps/desktop test -- status/no-query-tool` |
| 22 | Visual check both views against `screenshots/01-inbox.png` and `02-status.png` | 12,21 | `pnpm -C apps/desktop test -- visual -- inbox status` |
