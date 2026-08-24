# workshop-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Build one extension-editor frame parameterized by all nine extension types | — | `pnpm -C apps/desktop test -- workshop/shared-extension-editor` |
| 2 | Render the shared left rail, manual Save center, and version history | 1 | `pnpm -C apps/desktop test -- workshop/extension-editor-frame` |
| 3 | Centralize frontmatter field-kind inference for text, select, number, toggle, and chips | 1 | `pnpm -C apps/desktop test -- workshop/frontmatter-field-kinds` |
| 4 | Derive materialization totals and segment bars from the registry | 1 | `pnpm -C apps/desktop test -- workshop/materialization-from-registry` |
| 5 | Omit the Materialization rail for Harnesses only | 4 | `pnpm -C apps/desktop test -- workshop/harnesses-no-materialization` |
| 6 | Enforce lazy skill budget and its description-to-context downgrade | — | `cargo test -p locus-core materialize::skill_budget_and_downgrade` |
| 7 | Enforce one-glob rules and priority ordering | — | `cargo test -p locus-core materialize::rules_one_glob_and_priority` |
| 8 | Enforce exactly one native project context and surface its budget overflow as an upstream downgrade | — | `cargo test -p locus-core materialize::project_context_singleton` |
| 9 | Require command arguments and downgrade commands to skills with lost validation | — | `cargo test -p locus-core materialize::command_arguments_and_downgrade` |
| 10 | Model hooks as one event, threshold, timeout, and fail-open logging | — | `cargo test -p locus-core materialize::hook_contract` |
| 11 | Enforce one active output style per harness and role-scoped styles | — | `cargo test -p locus-core materialize::style_default_and_roles` |
| 12 | Keep linters outside materialization and persist warn/fail violations | — | `cargo test -p locus-core linters::not_materialized_and_violation_mode` |
| 13 | Rebuild the agent image only when its tool allowlist changes | — | `cargo test -p locus-core agents::tool_allowlist_rebuilds_image` |
| 14 | Render per-type invariant controls through the shared editor | 2,6,7,8,9,10,11,12,13 | `pnpm -C apps/desktop test -- workshop/extension-type-contracts` |
| 15 | Persist harness adapter config as adapter-owned JSONB | — | `cargo test -p locus-core harness::adapter_config_jsonb` |
| 16 | Restrict harness providers to configured Providers records | — | `cargo test -p locus-core harness::configured_provider_choices` |
| 17 | Keep adapter-less harnesses unselectable everywhere | 16 | `cargo test -p locus-core harness::adapter_unavailable` |
| 18 | Narrow routing default and band effort to the four shared values | — | `cargo test -p locus-core routing::effort_vocabulary` |
| 19 | Add the four-value database check for `routing_effort` | 18 | `cargo test -p locus-core store::routing_effort_constraint` |
| 20 | Retain upward model fallback when a routing band has no model | 18 | `cargo test -p locus-core routing::unconfigured_band_falls_upward` |
| 21 | Render Harnesses record, adapter-config table, and autorouting switch | 5,15,16,17,18 | `pnpm -C apps/desktop test -- workshop/harnesses` |
| 22 | Render the six-band table with only four effort choices | 18,20,21 | `pnpm -C apps/desktop test -- workshop/harnesses-routing-bands` |
| 23 | Derive provider `ok`, `warn`, and `off` status from verification plus expiry/staleness | — | `cargo test -p locus-core providers::status_projection` |
| 24 | Persist only a provider keychain reference and never a raw secret | — | `cargo test -p locus-core providers::keychain_reference_only` |
| 25 | Project preferred models, aliases, selector preview, and provider-removal model reset | 16,24 | `cargo test -p locus-core providers::selector_and_removal_projection` |
| 26 | Render Providers authentication, verification status, preferred models, and spend rail | 23,24,25 | `pnpm -C apps/desktop test -- workshop/providers` |
| 27 | Reject an uploaded tool with a missing, invalid, or untrusted Minisign signature before catalog admission | — | `cargo test -p locus-core tools::unsigned_upload_is_rejected` |
| 28 | Keep a rejected upload out of every image and render a signing remedy, never a read-only-role exception | 27 | `cargo test -p locus-core tools::rejected_upload_is_not_in_image` |
| 29 | Render grouped built-in tools, tri-state masters, uploaded-tool dropzone, rejection remedy, image, and usage rail | 27,28 | `pnpm -C apps/desktop test -- workshop/cli` |
| 30 | Replace the Goal canvas node with Governance goal and preserve existing graph validation | — | `cargo test -p locus-core workflow::goal_is_governance_not_node` |
| 31 | Share the closed condition-operand registry between UI and core validation | 30 | `cargo test -p locus-core workflow::condition_operands_are_closed` |
| 32 | Route human success criteria to an evidence-carrying inbox gate only | 30 | `cargo test -p locus-core workflow::human_criterion_is_gate` |
| 33 | Reinject governance guardrails after each reset | 30 | `cargo test -p locus-core workflow::guardrails_reinjected_after_reset` |
| 34 | Assert serialized Visual and Governance definitions carry no execution results | 30 | `cargo test -p locus-core workflow::authoring_has_no_run_state` |
| 35 | Render the Workflows list, autosave header, Visual/Governance switch, and no Save button | 30 | `pnpm -C apps/desktop test -- workshop/workflow-header-and-list` |
| 36 | Render the six-node palette, editable presets, condition inspector, and no Goal node | 30,31,35 | `pnpm -C apps/desktop test -- workshop/workflow-visual` |
| 37 | Render Governance goal, guardrails, and success-criteria editor | 30,32,33,35 | `pnpm -C apps/desktop test -- workshop/workflow-governance` |
| 38 | Add supersession pointers to `desktop-workshop-runtime/spec.md` and the Goal-node scope in `workflow-canvas/spec.md` | — | `grep -q "workshop-revision" .specs/{desktop-workshop-runtime,workflow-canvas}/spec.md` |
| 39 | Remove `minimal` from every effort selector, fixture, and database acceptance | 18,19,22 | `! grep -rq "minimal" apps/desktop/src crates/locus-core migrations/0017_autorouting_decisions.up.sql` |
| 40 | Assert Workshop routes all twelve views through the shared locator resolver | 14,21,26,29,35 | `pnpm -C apps/desktop test -- nav/workshop-locators` |
