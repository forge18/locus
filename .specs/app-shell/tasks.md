# app-shell — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `<Shell>` — 1440x900, radius 11, column flex, the window shadow | — | `pnpm -C apps/desktop test -- shell/frame` |
| 2 | `<TitleBar>` at 38px with traffic lights and the LOCUS wordmark | 1 | `pnpm -C apps/desktop test -- shell/titlebar` |
| 3 | `<LocatorBar>` 520x24 rendering `locus://` + path, with the `Cmd-K` affordance | 2 | `pnpm -C apps/desktop test -- shell/locator-bar` |
| 4 | `<ProjectFilter>` — multi-select scope control defaulting to all | 2 | `pnpm -C apps/desktop test -- shell/project-filters-not-switches` |
| 5 | Running count + pulsing accent dot, right side of the title bar | 2 | `pnpm -C apps/desktop test -- shell/running-count` |
| 6 | `<Rail>` at 78px with the seven category items and their Phosphor glyphs | 1 | `pnpm -C apps/desktop test -- shell/rail-seven-items` |
| 7 | Active rail state: `#293947` + accent inset ring + accent text | 6 | `pnpm -C apps/desktop test -- shell/rail-active` |
| 8 | Rail lights by category, not view — a drill-down keeps its parent lit | 6 | `pnpm -C apps/desktop test -- shell/rail-lights-by-category` |
| 9 | Inbox badge pill, absolutely positioned, hidden at zero | 6 | `pnpm -C apps/desktop test -- shell/inbox-badge` |
| 10 | Rail foot glyphs in `--mu2` | 6 | `pnpm -C apps/desktop test -- shell/rail-foot` |
| 11 | `<TabBar>` at 36px with the category label and the gradient ground | 1 | `pnpm -C apps/desktop test -- shell/tabbar` |
| 12 | Tab bar shows only the current category's tabs; three categories show none | 11 | `pnpm -C apps/desktop test -- shell/tabs-per-category` |
| 13 | Tab bar right side: the mono view locator + `arrows-out-simple` | 11 | `pnpm -C apps/desktop test -- shell/tabbar-locator` |
| 14 | `<Strip>` at 46px with the vertical label and one card per running agent | 1 | `pnpm -C apps/desktop test -- shell/strip` |
| 15 | Strip card: project · agent · role over status · tool · tokens | 14 | `pnpm -C apps/desktop test -- shell/strip-card` |
| 16 | Stuck cards get a red border; human-shell cards dim with a `terminal-window` icon | 15 | `pnpm -C apps/desktop test -- shell/strip-variants` |
| 17 | Sort needs-attention first then activity, asserted where the two orders differ | 15 | `pnpm -C apps/desktop test -- shell/strip-ordering` |
| 18 | Strip persists across every category change | 14 | `pnpm -C apps/desktop test -- shell/strip-persists` |
| 19 | Compose all four bands into `App.tsx`, replacing the Tauri scaffold | 2,6,11,14 | `pnpm -C apps/desktop build` |
| 20 | Assert the scaffold is gone — no greet handler, no Vite/Tauri logos | 19 | `bash apps/desktop/scripts/check-scaffold-removed.sh` |
