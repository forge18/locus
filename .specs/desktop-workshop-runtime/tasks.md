# desktop-workshop-runtime — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add provider identity and keychain-reference schema | — | `cargo test -p locus-core provider::reference_schema` |
| 2 | Add OS-keychain read/write/delete broker adapter | 1 | `cargo test -p locus-core provider::keychain_broker` |
| 3 | Redact provider secrets from rows, events, logs, and errors | 2 | `cargo test -p locus-core provider::never_persists_secret` |
| 4 | Resolve provider access only at the host egress broker | 2 | `cargo test -p locus-core provider::broker_only_access` |
| 5 | Reject provider access from an agent container | 4 | `cargo test -p locus-core provider::container_has_no_secret` |
| 6 | Persist authentication method and optional base URL | 1 | `cargo test -p locus-core provider::connection_config` |
| 7 | Verify a provider connection through the broker | 4,6 | `cargo test -p locus-core provider::verifies_connection` |
| 8 | Persist verification timestamp, model count, and failure state | 7 | `cargo test -p locus-core provider::verification_metadata` |
| 9 | Persist provider model catalog entries | 1 | `cargo test -p locus-core provider::model_catalog` |
| 10 | Persist aliases and selector inclusion | 9 | `cargo test -p locus-core provider::model_aliases` |
| 11 | Render Providers list, auth form, and connection status | 6,8,10 | `pnpm -C apps/desktop test -- workshop/providers` |
| 12 | Render preferred-model table and selector preview | 10 | `pnpm -C apps/desktop test -- workshop/provider-selector` |
| 13 | Add adapter registry identity/version schema | — | `cargo test -p locus-core adapter::registry` |
| 14 | Reject selecting a harness without an adapter | 13 | `cargo test -p locus-core harness::adapter_gate` |
| 15 | Persist harness-provider compatibility | 13 | `cargo test -p locus-core harness::provider_compatibility` |
| 16 | Reject a project harness with no configured compatible provider | 15 | `cargo test -p locus-core harness::project_provider_gate` |
| 17 | Persist default model and effort per harness | 15 | `cargo test -p locus-core harness::defaults` |
| 18 | Persist six complexity-band routing entries | 17 | `cargo test -p locus-core routing::six_bands` |
| 19 | Route missing bands upward and record fallback | 18 | `cargo test -p locus-core routing::falls_up` |
| 20 | Hold approval-ticked routing bands before start | 18 | `cargo test -p locus-core routing::approval_band` |
| 21 | Record selected provider, alias, model, effort, and band on run | 19,20 | `cargo test -p locus-core routing::decision_recorded` |
| 22 | Render Harnesses record and adapter config table | 17 | `pnpm -C apps/desktop test -- workshop/harness-record` |
| 23 | Render autorouting switch and six-band editor | 18 | `pnpm -C apps/desktop test -- workshop/autorouting` |
| 24 | Add trusted Minisign public-key store | — | `cargo test -p locus-core tools::trusted_keys` |
| 25 | Verify a tool manifest signature | 24 | `cargo test -p locus-core tools::manifest_signature` |
| 26 | Verify a tool binary digest/signature before catalog admission | 25 | `cargo test -p locus-core tools::binary_verification` |
| 27 | Reject unsigned or untrusted uploads | 26 | `cargo test -p locus-core tools::rejects_untrusted` |
| 28 | Persist built-in and verified user-tool metadata | 26 | `cargo test -p locus-core tools::catalog` |
| 29 | Persist category/group enablement and mixed state | 28 | `cargo test -p locus-core tools::group_toggles` |
| 30 | Resolve enabled tools into deterministic image set | 29 | `cargo test -p locus-core tools::image_set` |
| 31 | Rebuild an image only when its resolved tool set changes | 30 | `cargo test -p locus-core sandbox::tool_set_rebuild` |
| 32 | Render CLI categories, toggles, image details, and call counts | 29,31 | `pnpm -C apps/desktop test -- workshop/cli` |
| 33 | Render signed upload, verification result, and rejection state | 27 | `pnpm -C apps/desktop test -- workshop/cli-upload` |
| 34 | Render Agents, Commands, Hooks, Linters, Styles, Rules, and Skills views | — | `pnpm -C apps/desktop test -- workshop/extensions-desktop` |
| 35 | Render Workflow list states and authoring metadata | — | `pnpm -C apps/desktop test -- workflows/list` |
| 36 | Persist versioned workflow Governance root | 35 | `cargo test -p locus-core workflow::governance_root` |
| 37 | Persist goal independently from graph nodes | 36 | `cargo test -p locus-core workflow::goal_not_node` |
| 38 | Persist named guardrail prompt cards | 36 | `cargo test -p locus-core workflow::guardrail_prompts` |
| 39 | Persist typed success criteria and checker identity | 36 | `cargo test -p locus-core workflow::success_criteria` |
| 40 | Compile graph and Governance atomically | 37,38,39 | `cargo test -p locus-core workflow::governance_compiles` |
| 41 | Record governance evaluation on runs, not definitions | 40 | `cargo test -p locus-core workflow::results_on_run` |
| 42 | Render Visual canvas without goal/run state | 35,37 | `pnpm -C apps/desktop test -- workflows/visual` |
| 43 | Render Governance goal, guardrail, and criterion editors | 38,39 | `pnpm -C apps/desktop test -- workflows/governance` |
| 44 | Autosave authoring edits and expose save state | 42,43 | `pnpm -C apps/desktop test -- workflows/autosave` |
| 45 | Add Workshop loading, empty, error, keyboard, and theme fixtures | 11,22,32,34,42,43 | `pnpm -C apps/desktop test -- workshop/state-families` |
| 46 | Capture Dark/Light visual regressions for Workshop routes | 45 | `pnpm -C apps/desktop test -- visual/desktop-workshop` |
| 47 | Render desktop Agent definitions list, version editor, and materialization summary | 34 | `pnpm -C apps/desktop test -- workshop/agents-desktop` |
| 48 | Render Commands list, argument editor, and invocation preview | 34 | `pnpm -C apps/desktop test -- workshop/commands-desktop` |
| 49 | Render Hooks event mapping, consent, and delivery diagnostics | 34 | `pnpm -C apps/desktop test -- workshop/hooks-desktop` |
| 50 | Render Linters directory scope, rule text, command, and accent-budget evidence | 34 | `pnpm -C apps/desktop test -- workshop/linters-desktop` |
| 51 | Render Output Styles active-role mapping and fallback state | 34 | `pnpm -C apps/desktop test -- workshop/output-styles-desktop` |
| 52 | Render Rules scope, precedence, and materialization preview | 34 | `pnpm -C apps/desktop test -- workshop/rules-desktop` |
| 53 | Render Skills catalog, invocation metadata, and selected-skill detail | 34 | `pnpm -C apps/desktop test -- workshop/skills-desktop` |
