# model-resource-signal — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add `ContextOccupancy` and `ResourcePressure` types with checked approximate-remaining arithmetic, ordered context thresholds, and missing/contradictory → unknown validation | — | `cargo test -p locus-core model_resource_signal::occupancy_shape` |
| 2 | Read only the latest completed call's normalized ACP `usage.input`, label it last-observed, never sum prior usage, and render unknown when the latest call or model limit is missing | 1 | `cargo test -p locus-core model_resource_signal::normalized_acp_occupancy` |
| 3 | Add migrations for versioned model rate cards, cost observations, budget policies, and the idempotent budget ledger; all money is integer micro-USD | — | `cargo test -p locus-core model_resource_signal::schema` |
| 4 | Add Settings rate-card service and store operations for input/output/cache-read/cache-write prices per million tokens and pin the resolved version at dispatch | 3 | `cargo test -p locus-core model_resource_signal::rate_card_pinned` |
| 5 | Value one usage event: prefer provider-reported cost, otherwise use the pinned rate card and normalized ACP usage; missing usage/rate stays unknown | 4 | `cargo test -p locus-core model_resource_signal::cost_valuation` |
| 6 | Persist each cost observation once by `(run_id, seq)` and prove replay cannot double-spend an event | 5 | `cargo test -p locus-core model_resource_signal::cost_ledger_idempotent` |
| 7 | Add user-defined budget policy for run/task/project-day/global-day: optional micro-USD limit, warning %, acting %, notify/pause/cancel, unknown-cost action, and IANA billing timezone; reject incomplete or unordered settings | 3 | `cargo test -p locus-core model_resource_signal::budget_policy_validation` |
| 8 | Aggregate run spend from the idempotent ledger, preserving unknown when any required observation is unknown | 6,7 | `cargo test -p locus-core model_resource_signal::run_budget` |
| 9 | Aggregate root-task spend across every run and nested-agent descendant; taskless runs omit only the task scope | 6,7 | `cargo test -p locus-core model_resource_signal::task_budget_lineage` |
| 10 | Aggregate project-day and global-day spend by the Settings billing timezone with no project/day leakage and correct 23/25-hour daylight-saving days | 6,7 | `cargo test -p locus-core model_resource_signal::daily_budgets` |
| 11 | Resolve simultaneous scope states from one snapshot; record every crossing and apply the strictest action (`cancel > pause > notify`) without a looser scope weakening another | 8,9,10 | `cargo test -p locus-core model_resource_signal::strictest_action_wins` |
| 12 | Enforce notify/pause/cancel and unknown-cost actions at outer-turn/run boundaries; current turns finish, affected future turns/starts are blocked, and scope attribution is recorded | 11 | `cargo test -p locus-core model_resource_signal::outer_turn_enforcement` |
| 13 | Supersede the unimplemented token ceiling with the cost-only run scope; preserve every non-null legacy token value for user review and never reinterpret it as USD | 7 | `cargo test -p locus-core model_resource_signal::legacy_token_budget_preserved` |
| 14 | Materialize the frozen CTX~/BUD legend once near the beginning; state previous-call semantics and ceilings-not-targets, assert byte identity, and keep mutable values out | 1,7 | `cargo test -p locus-core model_resource_signal::frozen_legend` |
| 15 | Render `CTX~117k/200k; R~74k; N` from normalized ACP usage and `CTX U` when unavailable; ASCII only, deterministic rounding, at most 32 bytes | 2 | `cargo test -p locus-core model_resource_signal::ctx_renderer` |
| 16 | Render active cost scopes (`BUD$ r1.20/2H t4.80/10N p18/25H d31/50N D6h`), omitted disabled scopes, per-scope unknowns, deterministic money/reset rounding, at most 96 bytes | 8,9,10,11 | `cargo test -p locus-core model_resource_signal::budget_renderer` |
| 17 | Inject CTX~/BUD after recitation at the next outer-turn boundary, replace only the mutable-tail view, emit on pressure transitions, suppress unchanged rounded state, and preserve frozen-head bytes | 14,15,16 | `cargo test -p locus-core model_resource_signal::tail_injection` |
| 18 | Record rendered lines, exact producing snapshots, and active actions in the existing injection/materialization ledger; extend `context_attribution` without adding a telemetry verb | 17 | `cargo test -p locus-core model_resource_signal::injection_attribution` |
| 19 | Add `locus usage --json` for exact last-observed input, measurement source, every active spend/limit, thresholds, actions, and daily reset; unknown numbers serialize as `null` | 2,11 | `cargo test -p locus-cli usage::json` |
| 20 | Add live Settings commands and UI for rate cards, billing timezone, all four budget scopes, warning/acting percentages, threshold action, and unknown-cost action; no fixtures | 4,7 | `pnpm -C apps/desktop test -- settings/model-resource-budgets` |
| 21 | Run one shared normalized-ACP boundary fixture: reported input shows CTX~, missing input shows `CTX U`, nested task spend reaches task/project/day scopes, configured outer-turn action fires once, and replay is stable | 12,17,18,19 | `cargo test -p locus-core model_resource_signal::end_to_end` |

## Notes

- There is no per-harness capability matrix, implementation branch, calibration campaign,
  or harness-by-harness test suite. Every harness feeds the existing normalized ACP path;
  absent usage degrades to `CTX U`.
- Tasks 3–13 amend the still-authoritative guardrail engine contract; the old 85% token
  pause is not implemented and is replaced by user-configured cost policy rather than
  carried forward as a hidden default.
- Cost enforcement is honest about control granularity: work inside an active outer turn
  can overshoot. Task 12 blocks only at the boundary Locus already owns and must never
  claim provider-level hard-cap semantics.
- Task 10 uses a local calendar day in the configured IANA timezone, not a rolling 24-hour
  window.
- Task 17 reuses the context-layer recitation injection path and its 100ms/exit-0
  discipline. The persisted ledger is append-only even though the assembled tail view
  replaces its prior status lines.
- Task 20 follows the live command/provider seam from `desktop-data-integration`; it does
  not add another fixture-backed Settings path.
- Normal verification after the selected batch: `cargo test -p locus-core model_resource_signal && cargo test -p locus-cli usage::json && pnpm -C apps/desktop test -- settings/model-resource-budgets`.
