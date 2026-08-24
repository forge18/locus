# shell-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Register the nine-category rail model — five Project categories, four Cross-Project — replacing the seven-category union | — | `pnpm -C apps/desktop test -- shell/rail-nine-categories` |
| 2 | Purge retired category names from shell fixtures, route ids, and component names | 1 | `! grep -rqE "\\b($(printf '\\x44evelop | \\x41utomate | \\x44ashboard'))\\b" apps/desktop/src/shell apps/desktop/src/screens` |
| 3 | Build the 42px title bar — traffic lights, `LOCUS` wordmark, category/view label slot | 1 | `pnpm -C apps/desktop test -- shell/desktop-titlebar` |
| 4 | Render the current category and view label in the title bar | 3 | `pnpm -C apps/desktop test -- shell/desktop-title-labels` |
| 5 | Add the Dispatch pill — running count, pulsing dot while a run is active | 3 | `pnpm -C apps/desktop test -- shell/dispatch-dot` |
| 6 | Open and close the Dispatch activity popover with Attention needed / All tabs | 5 | `pnpm -C apps/desktop test -- shell/dispatch-popover-tabs` |
| 7 | Render Dispatch popover rows — icon, title, elapsed, project tag, meta | 6 | `pnpm -C apps/desktop test -- shell/dispatch-popover-rows` |
| 8 | Render the two Dispatch tab footer copy variants verbatim | 6 | `pnpm -C apps/desktop test -- shell/dispatch-popover-copy` |
| 9 | Render the Dispatch popover footer — Stop all, Open Dispatch — and route Open Dispatch to `autorun` | 6 | `pnpm -C apps/desktop test -- shell/dispatch-popover-footer` |
| 10 | Add the Inbox pill — tray icon, count badge | 3 | `pnpm -C apps/desktop test -- shell/inbox-badge` |
| 11 | Open and close the Inbox quick-preview popover and route Open Inbox to `inbox` | 10 | `pnpm -C apps/desktop test -- shell/inbox-popover` |
| 12 | Add the 212px rail with Project and Cross-Project groups | 1 | `pnpm -C apps/desktop test -- shell/project-rail` |
| 13 | Render the project switcher — type-to-filter, match highlighting | 12 | `pnpm -C apps/desktop test -- shell/project-switcher-keyboard` |
| 14 | Render the per-project running/spend note and the `+ New project` row | 13 | `pnpm -C apps/desktop test -- shell/project-switcher` |
| 15 | Render the Setup / Plan / Manage / Interact / Review project links | 12 | `pnpm -C apps/desktop test -- shell/project-rail-links` |
| 16 | Render the Analytics and Settings Cross-Project links | 12 | `pnpm -C apps/desktop test -- shell/global-rail-items` |
| 17 | Add the Memory expander — Short-term, Long-term, Artifacts, Wiki | 12 | `pnpm -C apps/desktop test -- shell/memory-expander` |
| 18 | Add the Workshop expander — eleven items, Agents first | 12 | `pnpm -C apps/desktop test -- shell/workshop-expander` |
| 19 | Persist rail expansion state across restart | 17,18 | `pnpm -C apps/desktop test -- shell/rail-expansion-persists` |
| 20 | Route every rail category to its landing view on first click | 15,16,17,18 | `pnpm -C apps/desktop test -- nav/category-landing-view` |
| 21 | Assert Dispatch and Inbox views are unreachable from the rail | 20 | `pnpm -C apps/desktop test -- nav/pill-only-views` |
| 22 | Route the rail, both pills, and the palette through one locator resolver | 9,11,20 | `pnpm -C apps/desktop test -- nav/one-resolver` |
| 23 | Add the ⌘K locator palette — Needs you / Running now / Where you were | 22 | `pnpm -C apps/desktop test -- nav/palette-results` |
| 24 | Render the palette footer copy and key hints | 23 | `pnpm -C apps/desktop test -- nav/palette-footer` |
| 25 | Add the toast stack, dismiss, and suppression on Interact and while the Dispatch popover is open | 3,6 | `pnpm -C apps/desktop test -- shell/toast-suppression` |
| 26 | Build the shared Merge modal — branch, commit split, both evidence columns | — | `pnpm -C apps/desktop test -- shell/merge-modal-columns` |
| 27 | Add the Merge modal warning box and both actions | 26 | `pnpm -C apps/desktop test -- shell/merge-modal-actions` |
| 28 | Build the Inbox To do / Completed tabs, throughput strip, and per-view project filter | 11 | `pnpm -C apps/desktop test -- inbox/desktop-groups` |
| 29 | Render the Inbox list as an `aria-live` log over the three item types | 28 | `pnpm -C apps/desktop test -- a11y/inbox-live-region` |
| 30 | Group Completed items by day with a time-to-resolve value | 28 | `pnpm -C apps/desktop test -- inbox/resolved` |
| 31 | Build the Gate detail pane — fields, plan body, comment box, both actions, both footnotes | 28 | `pnpm -C apps/desktop test -- inbox/gate-actions` |
| 32 | Add the "Superseded by" pointer line to `desktop-application-shell/spec.md` | — | `grep -q "Superseded by" .specs/desktop-application-shell/spec.md` |
