# dispatch-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add project archival state needed to lock autorun off | — | `cargo test -p locus-core dispatch::archived_project_cannot_autorun` |
| 2 | Replace the bare autorun boolean with `on`, `off`, and `suspended` states | 1 | `cargo test -p locus-core dispatch::autorun_state_distinguishes_manual_off_from_suspension` |
| 3 | Derive the All-projects master label and eligible count from project states | 1,2 | `cargo test -p locus-core dispatch::autorun_master_state` |
| 4 | Calculate rolling verify pass rate over the configured window | — | `cargo test -p locus-core dispatch::rolling_verify_pass_rate` |
| 5 | Auto-suspend below the 60% verify floor and resume on recovery | 2,4 | `cargo test -p locus-core dispatch::autorun_suspends_and_recovers` |
| 6 | Persist per-project review debt, pause threshold, inbox budget, and change ceiling | 1 | `cargo test -p locus-core dispatch::project_autorun_policy` |
| 7 | Pause autorun when unread landed artifacts exhaust review slots | 5,6 | `cargo test -p locus-core dispatch::review_debt_pauses_autorun` |
| 8 | Enforce the per-hour inbox budget before an autorun run enters the queue | 6 | `cargo test -p locus-core dispatch::autorun_inbox_budget` |
| 9 | Reject autorun work that touches `migrations/**` | — | `cargo test -p locus-core dispatch::autorun_rejects_migrations` |
| 10 | Reject autorun work whose workflow contains a Gate node | — | `cargo test -p locus-core dispatch::autorun_rejects_gate_workflows` |
| 11 | Reject autorun work over its resolved change ceiling | 6 | `cargo test -p locus-core dispatch::autorun_rejects_change_ceiling` |
| 12 | Reject autorun work under the verify floor and for the first task of a plan | 5 | `cargo test -p locus-core dispatch::autorun_rejects_untrusted_and_first_plan_tasks` |
| 13 | Route every autorun enqueue through all five fixed exclusions | 7,8,9,10,11,12 | `cargo test -p locus-core dispatch::autorun_exclusions_share_enqueue_boundary` |
| 14 | Extend Stop all with an optional bounded handoff write before stopping each run | — | `cargo test -p locus-core dispatch::stop_all_writes_handoffs_when_requested` |
| 15 | Preserve immediate stop semantics with no handoff row when the toggle is off | 14 | `cargo test -p locus-core dispatch::stop_all_immediate_without_handoffs` |
| 16 | Restore the exact saved autorun and schedule state for ten minutes | 14 | `cargo test -p locus-core dispatch::restore_stop_all_snapshot` |
| 17 | Make scheduled workflows nullable for Custom mode and add custom-run fields | — | `cargo test -p locus-core store::dispatch_custom_schedule_schema` |
| 18 | Persist optional per-schedule guardrail overrides and resolve unset fields from defaults | 17 | `cargo test -p locus-core dispatch::schedule_guardrail_fallthrough` |
| 19 | Project-mode schedules run active agents' assigned work and skip unassigned agents | 17 | `cargo test -p locus-core dispatch::project_schedule_skips_unassigned_agents` |
| 20 | Custom prompt-only schedules create a run and artifact, never a board task | 17 | `cargo test -p locus-core dispatch::custom_prompt_schedule_has_no_board_task` |
| 21 | Stop and split a schedule-originated run when its resolved ceiling is reached | 18 | `cargo test -p locus-core dispatch::schedule_ceiling_stops_and_splits` |
| 22 | Support Run once, scheduled, and Hold modes without queuing overlap | 17 | `cargo test -p locus-core dispatch::schedule_modes_and_overlap` |
| 23 | Resolve and test the Hourly, Nightly, Weekdays 09:00, and Once presets | 22 | `cargo test -p locus-core dispatch::cron_presets` |
| 24 | Flag a schedule that skips most recent firings and widen its interval | 22 | `cargo test -p locus-core dispatch::misconfigured_schedule_can_be_widened` |
| 25 | Project the full Runs verify vocabulary from run and waiting state | — | `cargo test -p locus-core dispatch::run_verify_vocabulary` |
| 26 | Persist stopping, change-size, and permission guardrail defaults | — | `cargo test -p locus-core store::guardrail_defaults_schema` |
| 27 | Permit a saved tighter default and require a recorded override for a looser one | 26 | `cargo test -p locus-core dispatch::guardrail_defaults_tighter_or_recorded_override` |
| 28 | Snapshot resolved defaults on run creation so later changes do not retune live runs | 26 | `cargo test -p locus-core dispatch::saved_defaults_do_not_retune_live_runs` |
| 29 | Render the Autorun master, project switches, archived lock, and suspended state | 3,5 | `pnpm -C apps/desktop test -- dispatch/autorun-switches` |
| 30 | Render review slots, review debt, pause threshold, inbox budget, and change ceiling | 6,7,8 | `pnpm -C apps/desktop test -- dispatch/autorun-review-debt` |
| 31 | Render the five Never-autoruns exclusions verbatim | 13 | `pnpm -C apps/desktop test -- dispatch/autorun-exclusions` |
| 32 | Render Stop all scope, handoff toggle, confirmation, result banner, and restore action | 14,15,16 | `pnpm -C apps/desktop test -- dispatch/stop-all` |
| 33 | Render the Schedules header and Start-work Project/Custom builder | 17,19,20 | `pnpm -C apps/desktop test -- dispatch/schedule-builder` |
| 34 | Render schedule guardrail overrides, resolved permission pills, and fallthrough copy | 18,21 | `pnpm -C apps/desktop test -- dispatch/schedule-guardrails` |
| 35 | Render When modes, cron readout, presets, and overlap-skipped copy | 22,23 | `pnpm -C apps/desktop test -- dispatch/schedule-when` |
| 36 | Render schedule cards, execution table, and misconfiguration banner | 22,24 | `pnpm -C apps/desktop test -- dispatch/schedule-results` |
| 37 | Render Runs search, sort, date range, and the three existing KPI projections | 25 | `pnpm -C apps/desktop test -- dispatch/runs-controls` |
| 38 | Render the full Runs table and verify vocabulary | 25,37 | `pnpm -C apps/desktop test -- dispatch/runs-table` |
| 39 | Render Settings → Guardrails' four groups and shipped-value reset | 26 | `pnpm -C apps/desktop test -- settings/guardrail-defaults` |
| 40 | Require explicit confirmation in Settings before persisting a looser default | 27,39 | `pnpm -C apps/desktop test -- settings/guardrail-looser-override` |
| 41 | Add supersession pointers to `schedules/spec.md` and `guardrails/spec.md` | — | `grep -q "Superseded by.*dispatch-revision" .specs/{schedules,guardrails}/spec.md` |
| 42 | Assert Autorun, Schedules, Runs, and Guardrails all route through the shared locator resolver | 29,33,37,39 | `pnpm -C apps/desktop test -- nav/dispatch-and-settings-locators` |
