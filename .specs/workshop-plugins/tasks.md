# workshop-plugins — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Define the common manifest envelope and plugin kind enum | — | `cargo test -p locus-core plugin::manifest_schema` |
| 2 | Implement JSON-RPC 2.0 stdio initialize/describe/health/shutdown | 1 | `cargo test -p locus-core plugin::lifecycle_roundtrip` |
| 3 | Negotiate protocol versions and reject malformed required capabilities | 2 | `cargo test -p locus-core plugin::capability_negotiation` |
| 4 | Bound plugin calls and preserve structured diagnostics on timeout | 2 | `cargo test -p locus-core plugin::call_timeout_is_bounded` |
| 5 | Keep plugin responses data-only and reject UI or persistence escape paths | 2 | `cargo test -p locus-core plugin::data_only_boundary` |
| 6 | Discover trusted user plugins without a core source change | 1,3 | `cargo test -p locus-core plugin::user_plugin_discovery` |
| 7 | Render Plugins and Extensions subgroup locators | — | `pnpm -C apps/desktop test -- workshop/plugin-extension-groups` |
| 8 | Keep the eight extension-editor types and Workflows under Extensions | 7 | `pnpm -C apps/desktop test -- workshop/extensions-scope` |
| 9 | Define the capability-based harness descriptor and optional capability rules | 1,3 | `cargo test -p locus-core plugin::harness_capabilities` |
| 10 | Adapt harness session events to the canonical ACP vocabulary | 9 | `cargo test -p locus-core plugin::harness_events_are_acp` |
| 11 | Register Pi as the only first-party harness plugin | 9,10 | `cargo test -p locus-core plugin::first_party_pi_only` |
| 12 | Define provider model, verification, alias, and keychain-reference capabilities | 1,3 | `cargo test -p locus-core plugin::provider_contract` |
| 13 | Register the OpenAI/ChatGPT provider plugin | 12 | `cargo test -p locus-core plugin::first_party_openai` |
| 14 | Register the Claude provider plugin | 12 | `cargo test -p locus-core plugin::first_party_anthropic` |
| 15 | Register the OpenRouter provider plugin | 12 | `cargo test -p locus-core plugin::first_party_openrouter` |
| 16 | Define CLI-tool install, verify, docs, digest, and permission capabilities | 1,3 | `cargo test -p locus-core plugin::cli_tool_contract` |
| 17 | Register GitHub CLI as the only first-party CLI tool | 16 | `cargo test -p locus-core plugin::first_party_gh_only` |
| 18 | Preserve Minisign admission and image allowlist gates for user CLI-tool plugins | 16 | `cargo test -p locus-core plugin::cli_tool_trust_boundary` |
| 19 | Reject non-first-party entries from the built-in catalog while accepting trusted user plugins | 6,11,12,16 | `cargo test -p locus-core plugin::built_in_allowlist` |
| 20 | Add the plugin contract suite for all three kinds | 5,9,12,16 | `cargo test -p locus-core plugin::contract_suite` |
