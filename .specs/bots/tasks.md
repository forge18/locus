# bots — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `bots` table binding project, definition, and home session | — | `cargo test -p locus-core bots::table` |
| 2 | Create a bot: definition plus row, unique name per project | 1 | `cargo test -p locus-core bots::create` |
| 3 | Home session on first message; `bots/<bot-id>` branch via the repo-manager clone model, never `main` | 2 | `cargo test -p locus-core bots::home_session` |
| 4 | Resume across runs: one conversation, cost summed | 3 | `cargo test -p locus-core bots::resumes_across_runs` |
| 5 | Warm window: idle timer takes the existing stop path; default 10m, project setting `bots.warm_window_minutes` | 3 | `cargo test -p locus-core bots::warm_expiry` |
| 6 | A warm stop loses nothing: message after expiry resumes the conversation and workspace | 5 | `cargo test -p locus-core bots::warm_resume` |
| 7 | Reconciliation treats a warm-stopped bot container as expected — no aborted-run inbox item | 5 | `cargo test -p locus-core bots::reconcile_warm_stop` |
| 8 | Routine: a schedule whose target is a prompt bound to a bot | 2 | `cargo test -p locus-core bots::routine_target` |
| 9 | `locusd` fires a routine headless, booting a cold home session | 8 | `cargo test -p locus-core bots::routine_fires_headless` |
| 10 | Record the execution with its result | 9 | `cargo test -p locus-core bots::routine_records_result` |
| 11 | Firing during a run is skipped and dropped, count visible, nothing queued | 9 | `cargo test -p locus-core bots::routine_skips_never_queues` |
| 12 | Routine turn lands in the conversation attributed as routine-fired | 9 | `cargo test -p locus-core bots::routine_attribution` |
| 13 | Pause, resume, edit, delete a routine, keeping history | 8 | `cargo test -p locus-core bots::routine_lifecycle` |
| 14 | Run records the definition version used; edits change the next run, never a running one | 3 | `cargo test -p locus-core bots::definition_version_per_run` |
| 15 | Bots rail category, `bots` view, both locators, resolver round-trip | — | `pnpm -C apps/desktop test -- bots/navigation` |
| 16 | Bot list rail: live dot, name, harness, last activity, New bot, verbatim footer and empty state | 15 | `pnpm -C apps/desktop test -- bots/list` |
| 17 | Bot view composes the Agent Pane unmodified against the home session | 16 | `pnpm -C apps/desktop test -- bots/panel` |
| 18 | Routines sheet: list, pause/enable, edit, delete, Test run marked as a test | 13, 17 | `pnpm -C apps/desktop test -- bots/routines-sheet` |
| 19 | design-revision inventory and vocabulary and `docs/UI_MOCKUP_REVIEW.md` updated to thirty views | 15 | `rg -n "bots" .specs/design-revision/spec.md docs/UI_MOCKUP_REVIEW.md` |
