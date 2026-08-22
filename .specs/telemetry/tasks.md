# telemetry — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Event type: the twelve verbs as a closed enum, `usage` optional | — | `cargo test -p locus-core telemetry::vocabulary_is_closed` |
| 2 | Assert a thirteenth verb fails to compile, not to run | 1 | `cargo test -p locus-core telemetry::no_extension_at_runtime` |
| 3 | `seq` assigned at the core, total per run | 1 | `cargo test -p locus-core telemetry::seq_is_total` |
| 4 | `acp` adapter — one mapping for every ACP harness | 1 | `cargo test -p locus-core telemetry::acp_adapter` |
| 5 | `mcpServers` empty on every ACP call | 4 | `cargo test -p locus-core telemetry::mcp_always_empty` |
| 6 | A harness with no native ACP mode is bridged by a Locus-side mapping registered per harness | 4 | `cargo test -p locus-core telemetry::bridge_registered_per_harness` |
| 7 | Assert the `acp` source is the only telemetry source — no hooks/stream/session-log path remains | 4 | `cargo test -p locus-core telemetry::single_source` |
| 8 | Persist `raw` JSONB on every event | 1 | `cargo test -p locus-core telemetry::raw_always_present` |
| 9 | Replay a run against a fixed parser without re-running the agent | 8 | `cargo test -p locus-core telemetry::replay_repairs` |
| 10 | `usage` null rather than zero where unreported | 1 | `cargo test -p locus-core telemetry::usage_unknown_not_zero` |
| 11 | Assert Locus never computes a token count itself | 10 | `cargo test -p locus-core telemetry::never_counts_tokens` |
| 12 | A missing verb is absent, never synthesized empty | 1 | `cargo test -p locus-core telemetry::missing_verb_stays_missing` |
| 13 | Per-harness declared verb set drives what a test expects | 12 | `cargo test -p locus-core telemetry::expectations_per_harness` |
| 14 | `permission_request` raises an alarm | 1 | `cargo test -p locus-core telemetry::permission_request_alarms` |
| 15 | One ACP source yields the shared downstream shape for every harness | 4 | `cargo test -p locus-core telemetry::single_source_shape` |
| 16 | Event-based test harness: "run this, assert these events" | 15 | `cargo test -p locus-core telemetry::event_assertions` |
| 17 | Stream normalized events to the UI over `Channel<Event>` | 15 | `pnpm -C apps/desktop test -- transcript/from-core` |
| 18 | `stream_pos` assigned at the core from one monotonic counter, total per project across runs | 3 | `cargo test -p locus-core telemetry::stream_pos_is_total_per_project` |
| 19 | Two concurrent runs: union of polls at `stream_pos > watermark` equals every event written | 18 | `cargo test -p locus-core telemetry::cursor_never_skips` |
