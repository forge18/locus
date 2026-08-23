# design-desktop — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Register all 31 desktop screen routes in the fixture inventory | — | `pnpm -C apps/desktop test -- fixtures/desktop-screen-inventory` |
| 2 | Replace the v1 shell fixture with title bar, project rail, selected-project card, and active-session popover | 1 | `pnpm -C apps/desktop test -- shell/desktop-project-rail` |
| 3 | Apply desktop semantic tokens and prohibit accent-backed magnitude charts | 1 | `pnpm -C apps/desktop test -- design/desktop-accent-roles` |
| 4 | Add the global/project-scoped navigation resolver contract | 2 | `pnpm -C apps/desktop test -- nav/desktop-project-scope` |
| 5 | Render Inbox and Dashboard desktop fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-dashboard` |
| 6 | Render Project Settings and Analytics fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-projects` |
| 7 | Render Plan Conversation, Spec, and Tasks & cards fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-plan` |
| 8 | Render Develop, Automate, and Review fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-project-work` |
| 9 | Render Dispatch Autorun, Schedules, and Runs fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-dispatch` |
| 10 | Render Memory Short-term, Long-term, Artifacts, and Wiki fixtures | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-memory` |
| 11 | Render Settings Guardrails fixture | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-settings` |
| 12 | Render all Workshop fixtures, including CLI and Providers | 2,3 | `pnpm -C apps/desktop test -- screens/desktop-workshop` |
| 13 | Add provider reference schema, OS-keychain broker boundary, and redaction tests | — | `cargo test -p locus-core provider::never_persists_secret` |
| 14 | Add provider model catalogs, aliases, verification metadata, and selector projection | 13 | `cargo test -p locus-core provider::selector_aliases` |
| 15 | Add adapter-gated project harness selection and provider compatibility | 13 | `cargo test -p locus-core harness::project_selection_gate` |
| 16 | Add six-band autorouting with upward fallback and recorded decision | 14,15 | `cargo test -p locus-core routing::falls_up_and_records` |
| 17 | Add trusted Minisign keys, signed user-tool verification, enabled catalog, and image-set resolution | — | `cargo test -p locus-core tools::minisign_verification` |
| 18 | Add project extension and role-scoped tool subtraction before materialization | 15,17 | `cargo test -p locus-core materialize::project_scope_subtracts` |
| 19 | Add editable plan decomposition and spec/task-to-card mapping | — | `cargo test -p locus-core planning::decomposes_to_cards` |
| 20 | Preserve dependencies and require final approval before cards are created | 19 | `cargo test -p locus-core planning::approval_commits_cards` |
| 21 | Add durable dispatch queue with global/per-project caps and priority policy | — | `cargo test -p locus-core dispatch::enforces_parallel_caps` |
| 22 | Add iteration-boundary preemption and handoff preservation | 21 | `cargo test -p locus-core dispatch::preempts_at_boundary` |
| 23 | Add Stop all snapshot, global autorun/schedule stop, and ten-minute restore | 21 | `cargo test -p locus-core dispatch::stop_all_restores` |
| 24 | Add workflow Governance schema and ensure authoring routes exclude run state | — | `cargo test -p locus-core workflow::governance_is_versioned && pnpm -C apps/desktop test -- workflow/governance` |
