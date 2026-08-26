# workshop-revision — tasks

This task list covers the **Extensions** subgroup only. Plugin subgroup work moved to
`.specs/workshop-plugins/tasks.md`; its first-party roster is `gh`, `pi`, `openai`, `anthropic`, and
`openrouter`, while user plugins remain supported through the common contract.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Build one extension-editor frame parameterized by the eight extension types | — | `pnpm -C apps/desktop test -- workshop/shared-extension-editor` |
| 2 | Render the shared left rail, manual Save center, and version history | 1 | `pnpm -C apps/desktop test -- workshop/extension-editor-frame` |
| 3 | Centralize frontmatter field-kind inference for text, select, number, toggle, and chips | 1 | `pnpm -C apps/desktop test -- workshop/frontmatter-field-kinds` |
| 4 | Derive materialization totals and segment bars from the registry | 1 | `pnpm -C apps/desktop test -- workshop/materialization-from-registry` |
| 5 | Keep plugin records outside the extension editor's Materialization rail | 4 | `pnpm -C apps/desktop test -- workshop/plugins-outside-extension-editor` |
| 6 | Enforce lazy skill budget and its description-to-context downgrade | — | `cargo test -p locus-core materialize::skill_budget_and_downgrade` |
| 7 | Enforce one-glob rules and priority ordering | — | `cargo test -p locus-core materialize::rules_one_glob_and_priority` |
| 8 | Enforce exactly one native project context and surface its budget overflow as an upstream downgrade | — | `cargo test -p locus-core materialize::project_context_singleton` |
| 9 | Require command arguments and downgrade commands to skills with lost validation | — | `cargo test -p locus-core materialize::command_arguments_and_downgrade` |
| 10 | Model hooks as one event, threshold, timeout, and fail-open logging | — | `cargo test -p locus-core materialize::hook_contract` |
| 11 | Enforce one active output style per harness and role-scoped styles | — | `cargo test -p locus-core materialize::style_default_and_roles` |
| 12 | Keep linters outside materialization and persist warn/fail violations | — | `cargo test -p locus-core linters::not_materialized_and_violation_mode` |
| 13 | Rebuild the agent image only when its tool allowlist changes | — | `cargo test -p locus-core agents::tool_allowlist_rebuilds_image` |
| 14 | Render per-type invariant controls through the shared editor | 2,6,7,8,9,10,11,12,13 | `pnpm -C apps/desktop test -- workshop/extension-type-contracts` |
| 15 | Render Plugins and Extensions as the two Workshop subgroups | 1,14 | `pnpm -C apps/desktop test -- workshop/plugin-extension-groups` |
| 16 | Render the Workflows list, autosave header, Visual/Governance switch, and no Save button | 15 | `pnpm -C apps/desktop test -- workshop/workflow-header-and-list` |
| 17 | Render the six-node palette, editable presets, condition inspector, and no Goal node | 16 | `pnpm -C apps/desktop test -- workshop/workflow-visual` |
| 18 | Render Governance goal, guardrails, and success-criteria editor | 16 | `pnpm -C apps/desktop test -- workshop/workflow-governance` |
| 19 | Preserve the closed condition-operand registry between UI and core validation | 17 | `cargo test -p locus-core workflow::condition_operands_are_closed` |
| 20 | Route human success criteria to an evidence-carrying inbox gate only | 16 | `cargo test -p locus-core workflow::human_criterion_is_gate` |
| 21 | Reinject governance guardrails after each reset | 16 | `cargo test -p locus-core workflow::guardrails_reinjected_after_reset` |
| 22 | Assert serialized Visual and Governance definitions carry no execution results | 16 | `cargo test -p locus-core workflow::authoring_has_no_run_state` |
| 23 | Replace the Goal canvas node with Governance goal and preserve existing graph validation | 17 | `cargo test -p locus-core workflow::goal_is_governance_not_node` |
| 24 | Add supersession pointers to historical Workshop specs and the Goal-node scope | — | `grep -q "workshop-plugins" .specs/{desktop-workshop-runtime,workflow-canvas}/spec.md` |
| 25 | Remove `minimal` from every effort selector, fixture, and database acceptance | — | `! grep -rq "minimal" apps/desktop/src crates/locus-core migrations/0017_autorouting_decisions.up.sql` |
| 26 | Route all Workshop views through the shared locator resolver | 15,16 | `pnpm -C apps/desktop test -- nav/workshop-locators` |
