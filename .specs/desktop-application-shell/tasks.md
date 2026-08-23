# desktop-application-shell — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Register global and project-scoped desktop route kinds | — | `pnpm -C apps/desktop test -- nav/desktop-route-kinds` |
| 2 | Add selected-project state with restart persistence | 1 | `pnpm -C apps/desktop test -- shell/project-persists` |
| 3 | Add explicit global-scope state | 1 | `pnpm -C apps/desktop test -- shell/global-scope` |
| 4 | Reject a project-only route without a selected project | 1 | `pnpm -C apps/desktop test -- nav/project-route-requires-project` |
| 5 | Build the 42px desktop title bar | — | `pnpm -C apps/desktop test -- shell/desktop-titlebar` |
| 6 | Render current category and view labels | 5 | `pnpm -C apps/desktop test -- shell/desktop-title-labels` |
| 7 | Render running and needs-you counts in the title pill | 5 | `pnpm -C apps/desktop test -- shell/running-pill-counts` |
| 8 | Open and close the active-session popover | 7 | `pnpm -C apps/desktop test -- shell/running-popover` |
| 9 | Sort popover runs by needs-attention then activity | 8 | `pnpm -C apps/desktop test -- shell/running-popover-order` |
| 10 | Add the 212px global/project rail | 2,3 | `pnpm -C apps/desktop test -- shell/project-rail` |
| 11 | Render global rail items and Inbox badge | 10 | `pnpm -C apps/desktop test -- shell/global-rail-items` |
| 12 | Render selected-project card and type-filter switcher | 10 | `pnpm -C apps/desktop test -- shell/project-switcher` |
| 13 | Highlight project-switcher matches and keyboard movement | 12 | `pnpm -C apps/desktop test -- shell/project-switcher-keyboard` |
| 14 | Render project-only Plan/Develop/Automate/Review links | 12 | `pnpm -C apps/desktop test -- shell/project-rail-links` |
| 15 | Render expandable Memory links | 10 | `pnpm -C apps/desktop test -- shell/memory-expander` |
| 16 | Render expandable Workshop links | 10 | `pnpm -C apps/desktop test -- shell/workshop-expander` |
| 17 | Persist rail expansion state | 15,16 | `pnpm -C apps/desktop test -- shell/rail-expansion-persists` |
| 18 | Render green/amber/red Dispatch dot states | 10 | `pnpm -C apps/desktop test -- shell/dispatch-dot` |
| 19 | Add locator and Cmd-K entry point | 1,5 | `pnpm -C apps/desktop test -- nav/desktop-locator` |
| 20 | Add recognition-first palette results | 19 | `pnpm -C apps/desktop test -- nav/palette-results` |
| 21 | Route palette, rail, and links through one resolver | 14,19,20 | `pnpm -C apps/desktop test -- nav/one-resolver` |
| 22 | Add per-window back and forward history | 21 | `pnpm -C apps/desktop test -- nav/window-history` |
| 23 | Add keyboard focus order and roving selection | 8,13,15,16,20 | `pnpm -C apps/desktop test -- shell/keyboard-navigation` |
| 24 | Add live-region politeness map | 7,18 | `pnpm -C apps/desktop test -- a11y/shell-live-regions` |
| 25 | Add loading, empty, and error shell states | 5,10 | `pnpm -C apps/desktop test -- shell/state-families` |
| 26 | Render shell in Dark and Light fixture suites | 5,10 | `pnpm -C apps/desktop test -- shell/themes` |
| 27 | Assert no v1 filter, tab bar, or strip renders | 10,21 | `pnpm -C apps/desktop test -- shell/no-v1-chrome` |
| 28 | Capture visual regression fixtures for shell states | 8,18,26 | `pnpm -C apps/desktop test -- visual/desktop-shell` |
