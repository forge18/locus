# v2-project-operations — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Persist project settings root | — | `cargo test -p locus-core project::settings_roundtrip` |
| 2 | Persist project harness allow-list | 1 | `cargo test -p locus-core project::harness_allow_list` |
| 3 | Persist exactly one agent default | 2 | `cargo test -p locus-core project::one_agent_default` |
| 4 | Render harness allow-list and router summary | 2,3 | `pnpm -C apps/desktop test -- projects/harnesses` |
| 5 | Persist repos and per-repo project membership | 1 | `cargo test -p locus-core project::repos` |
| 6 | Render repo rows and branch/run state | 5 | `pnpm -C apps/desktop test -- projects/repos` |
| 7 | Persist editable base context and token budget | 1 | `cargo test -p locus-core project::base_context` |
| 8 | Render base-context editor and budget meter | 7 | `pnpm -C apps/desktop test -- projects/base-context` |
| 9 | Persist extension group and item overrides | 1 | `cargo test -p locus-core project::extension_overrides` |
| 10 | Exclude disabled extensions from materialization | 9 | `cargo test -p locus-core materialize::disabled_extensions_absent` |
| 11 | Persist per-role CLI tool subtraction | 1 | `cargo test -p locus-core project::tool_scope` |
| 12 | Exclude scoped-out tools from launch image | 11 | `cargo test -p locus-core sandbox::role_tools_absent` |
| 13 | Aggregate project analytics from run data | 1 | `cargo test -p locus-core project::analytics` |
| 14 | Render model/token/cache/spend analytics table | 13 | `pnpm -C apps/desktop test -- projects/analytics` |
| 15 | Model planning stage state and nine-stage transitions | — | `cargo test -p locus-core planning::nine_stages` |
| 16 | Render conversation stage progress and live line | 15 | `pnpm -C apps/desktop test -- plan/conversation-v2` |
| 17 | Persist requirement IDs and editable spec blocks | 15 | `cargo test -p locus-core planning::editable_requirements` |
| 18 | Re-audit changed requirements only | 17 | `cargo test -p locus-core planning::reaudits_changed_requirements` |
| 19 | Render spec outline, finding, and unsaved state | 17,18 | `pnpm -C apps/desktop test -- plan/spec-editor` |
| 20 | Persist task estimates, roles, dependencies, and card mode | 17 | `cargo test -p locus-core planning::task_decomposition` |
| 21 | Calculate spec-only card mode | 20 | `cargo test -p locus-core planning::spec_only_cards` |
| 22 | Calculate every-task card mode | 20 | `cargo test -p locus-core planning::every_task_cards` |
| 23 | Calculate selected carve-out card mode | 20 | `cargo test -p locus-core planning::carve_out_cards` |
| 24 | Render editable Tasks & cards table and count | 20,21,22,23 | `pnpm -C apps/desktop test -- plan/tasks-cards` |
| 25 | Commit mapping only with final approval | 23 | `cargo test -p locus-core planning::approval_commits_mapping` |
| 26 | Create cards without losing dependency edges | 25 | `cargo test -p locus-core planning::cards_keep_dependencies` |
| 27 | Persist global/per-project parallel caps | 1 | `cargo test -p locus-core dispatch::parallel_caps` |
| 28 | Persist priority and tie-break policies | 27 | `cargo test -p locus-core dispatch::priority_policy` |
| 29 | Render Guardrails parallelism controls | 27,28 | `pnpm -C apps/desktop test -- settings/parallelism` |
| 30 | Queue work rather than exceed a cap | 27 | `cargo test -p locus-core dispatch::queues_at_cap` |
| 31 | Preempt only at an iteration boundary | 30 | `cargo test -p locus-core dispatch::preempts_at_boundary` |
| 32 | Preserve handoff and context policy on preemption | 31 | `cargo test -p locus-core dispatch::preemption_handoff` |
| 33 | Persist per-project autorun state | 1 | `cargo test -p locus-core dispatch::autorun_state` |
| 34 | Render Autorun rationale and review-debt state | 33 | `pnpm -C apps/desktop test -- dispatch/autorun` |
| 35 | Render schedules and overlap/skip outcome | 33 | `pnpm -C apps/desktop test -- dispatch/schedules` |
| 36 | Render queue/runs and pause controls | 30,33 | `pnpm -C apps/desktop test -- dispatch/runs` |
| 37 | Snapshot Stop all affected state | 30,33 | `cargo test -p locus-core dispatch::stop_all_snapshot` |
| 38 | Stop runs, autorun, and schedules without deleting durable work | 37 | `cargo test -p locus-core dispatch::stop_all_preserves_work` |
| 39 | Restore Stop all state within ten minutes | 38 | `cargo test -p locus-core dispatch::restore_window` |
| 40 | Render Stop all confirmation, stopped banner, and restore action | 37,38,39 | `pnpm -C apps/desktop test -- dispatch/stop-all` |
| 41 | Render project list with running/idle/archived state | 1 | `pnpm -C apps/desktop test -- projects/list` |
| 42 | Create, rename, and archive a project without rewriting historical rows | 1 | `cargo test -p locus-core project::lifecycle_preserves_history` |
| 43 | Render project Extensions groups and group toggles | 9 | `pnpm -C apps/desktop test -- projects/extensions-groups` |
| 44 | Render per-extension toggles and materialization consequence | 9,10 | `pnpm -C apps/desktop test -- projects/extension-toggle` |
| 45 | Render project CLI search-and-add and role scope controls | 11,12 | `pnpm -C apps/desktop test -- projects/cli-tools` |
| 46 | Render planning draft outputs, recommendation, and tools rail | 16,19 | `pnpm -C apps/desktop test -- plan/draft-outputs` |
| 47 | Render card-mode choices, row carve-out toggle, and approval count copy | 24 | `pnpm -C apps/desktop test -- plan/card-mode-controls` |
| 48 | Render selected-project Automate Kanban and fixed board columns | 25 | `pnpm -C apps/desktop test -- automate/kanban-v2` |
| 49 | Render task cards with role, dependencies, estimates, gate, and review state | 48 | `pnpm -C apps/desktop test -- automate/task-cards-v2` |
| 50 | Render Automate agents list with run status, transcript, and filters | 30 | `pnpm -C apps/desktop test -- automate/agents-v2` |
| 51 | Render agent handoff, pause, cancel, and needs-attention controls | 32,50 | `pnpm -C apps/desktop test -- automate/agent-controls` |
| 52 | Render all Guardrails controls: iterations, budget, stuck, reassignment, wall-clock, and parallelism | 27,28 | `pnpm -C apps/desktop test -- settings/guardrails-v2` |
