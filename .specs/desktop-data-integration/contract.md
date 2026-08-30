# Desktop live-data contract (frozen)

Task 1 of `tasks.md`. This is the single source of truth for which Rust command every
desktop accessor reads, and its scope. Task 2 builds the provider seam on this table;
tasks 3–9 migrate screens in this order. Update this table only with the task whose
slice changes it.

## Ground truth (measured 2026-08-30)

- `apps/desktop/src/data/` holds 23 accessor modules; 2 are fully live (`bots`,
  `work-items`), 21 return fixtures or mix fixtures with live calls.
- The accessors promise **82 distinct Tauri commands** via `Becomes:` markers.
- The host registers **46** commands; **8** of the promised 82 exist
  (`artifacts_list`, `artifact_comments`, `harness_tier_grid`, `materialization_report`,
  `projects_list`, `repos_list`, `local_remotes_list`, `project_setup` — the last four are the
  slice-3 tracer bullet). **74 commands are missing.**

## Scope legend

- **project** — takes a `projectId` (or an id whose row hangs off a project); every
  query and mutation must be proven isolated against a second project (guardrail).
- **global** — project-independent host data.

## Per-module contract

| Module | Commands (status) | Scope |
| --- | --- | --- |
| `bots.ts` | `bots_list`, `bot_create`, `bot_routines`, `bot_routine_executions`, `bot_routine_set_enabled`, `bot_routine_update`, `bot_routine_delete` — **live** | project (bot rows are project-scoped) |
| `work-items.ts` | `external_work_item_*` (13) — **live** | project |
| `core.ts` | `projects_list`, `repos_list`, `local_remotes_list` — **live** (slice 3) | `projects_list` global; `repos_list`/`local_remotes_list` project |
| `sessions.ts` | `sessions_list`, `session`, `runs_for_session` — **missing** | project |
| `runs.ts` | `runs_list`, `runs_page`, `runs_count`, `run_stats` — **missing** | project |
| `telemetry.ts` | `telemetry_metrics`, `telemetry_spend`, `telemetry_facets`, `telemetry_filters`, `telemetry_actions`, `telemetry_tools`, `telemetry_verb_counts`, `telemetry_sessions`, `telemetry_sessions_page`, `telemetry_sessions_count`, `sessions_over_time` — **missing** | project |
| `analytics.ts` | `analytics_at_a_glance`, `analytics_stats`, `analytics_breakdown`, `analytics_breakdown_dimensions`, `analytics_task_outcomes`, `analytics_workflow_timings`, `analytics_retrieval_tiers`, `analytics_extension_usage`, `analytics_extension_kinds`, `analytics_telemetry_facets`, `analytics_telemetry_actions`, `analytics_telemetry_sessions`, `analytics_telemetry_verbs` — **missing** | project |
| `plan.ts` | `plans_list`, `plan_outputs`, `plan_recommendation`, `plan_scope_decision` — **missing** | project |
| `knowledge.ts` | `memory_facts`, `memory_short_term`, `memory_compacted_artifacts` — **missing** | project |
| `mail.ts` | `mail_threads`, `mail_messages`, `mail_participants` — **missing** | project |
| `board.ts` | `board_tasks`, `board_dependencies`, `task_evidence` — **missing** | project |
| `qa.ts` | `qa_sources`, `qa_snapshot`, `qa_checks` — **missing** | project |
| `inbox.ts` | `inbox_list`, `inbox_resolved_today`, `inbox_throughput` — **missing** | project |
| `dispatch.ts` | `dispatch_autorun`, `dispatch_runs`, `dispatch_schedules`, `dispatch_schedule_executions` — **missing** (mutation `dispatch_stop_all` **live**) | project |
| `workflow.ts` | `workflow_presets`, `workflow_def`, `workflow_graph`, `workflow_guardrails`, `workflow_node_vocabulary`, `condition_operands`, `condition_expression` — **missing** | project |
| `workflow-events.ts` | re-exports the fixture literal; becomes a `Channel` subscription of `workflow.*` log events | project |
| `artifacts.ts` | `artifacts_list`, `artifact_comments` **live**; `artifact`, `artifact_diff`, `artifact_kinds` **missing** | project |
| `harnesses.ts` | `harness_tier_grid` **live**; `harness_registry_list`, `harness_registry_summary`, `extension_types` **missing** | global |
| `extensions.ts` | `extension_inventory` **missing** (shared with harnesses), `recently_edited` **missing** | global |
| `settings.ts` | `harness_tier_grid` **live**; `settings_model_tiers`, `resolve_model_tier`, `settings_tier_fallback` **missing** | global |
| `guardrails.ts` | `settings_guardrails` **missing** | global |
| `agent-defs.ts` | `agent_defs_list`, `agent_def` **live**; `materialization_report` **live** | global (defs are host-level) |
| `strip.ts` | `strip_cards`, `running_count` — **missing** | global (cross-project running view) |

## Setup policy and base context

`project_setup` (project) — added in slice 3, serves the Setup screen's harness policy
(`harnessAllowList`) and base context (`baseContext`, `baseContextTokenBudget`) from
`ProjectSettings`. It was not among the original `Becomes:` markers; the contract owns it now.

## Envelope

Every migrated accessor returns `Envelope<T>` from `apps/desktop/src/data/envelope.ts`
(`loading` | `empty` | `ready` | `failed`), never a bare array and never a fixture.
An IPC failure is `failed` with the command name — it is never swapped for a fixture,
an empty success, or a zero-valued metric.

## Fallback rule

Until a module's slice lands, its accessors may keep returning fixtures, but only
behind the task-2 provider seam. After a slice lands, the fixture path is reachable
exclusively through the explicit demo/test provider (task 10 deletes the rest).
