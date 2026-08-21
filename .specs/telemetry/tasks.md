# telemetry — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Event type: the twelve verbs as a closed enum, `usage` optional | — | `cargo test -p locus-core telemetry::vocabulary_is_closed` |
| 2 | Assert a thirteenth verb fails to compile, not to run | 1 | `cargo test -p locus-core telemetry::no_extension_at_runtime` |
| 3 | `seq` assigned at the core, total per run | 1 | `cargo test -p locus-core telemetry::seq_is_total` |
| 4 | `hooks` adapter with a per-harness event-name table | 1 | `cargo test -p locus-core telemetry::hooks_adapter` |
| 5 | `locus-hook` binary: JSON on stdin, exit 0 on every failure path | 4 | `cargo test -p locus-cli hook::always_exits_zero` |
| 6 | Hook logging appends to a local buffer, never the socket synchronously | 5 | `cargo test -p locus-cli hook::no_sync_socket` |
| 7 | Hook injection path carries a 100ms timeout and emits nothing on expiry | 5 | `cargo test -p locus-cli hook::injection_timeout` |
| 8 | `acp` adapter — one mapping for every ACP harness | 1 | `cargo test -p locus-core telemetry::acp_adapter` |
| 9 | `stream-json` adapter, driven by the TOML's type key and verb table | 1 | `cargo test -p locus-core telemetry::stream_json_adapter` |
| 10 | Tee stdout: same bytes to terminal and normalizer | 9 | `cargo test -p locus-core telemetry::tee_is_lossless` |
| 11 | `session-log` adapter with a per-harness parser | 1 | `cargo test -p locus-core telemetry::session_log_adapter` |
| 12 | Tail while live, re-read once at exit | 11 | `cargo test -p locus-core telemetry::session_log_reread` |
| 13 | Persist `raw` JSONB on every event | 1 | `cargo test -p locus-core telemetry::raw_always_present` |
| 14 | Replay a run against a fixed parser without re-running the agent | 13 | `cargo test -p locus-core telemetry::replay_repairs` |
| 15 | `usage` null rather than zero where unreported | 1 | `cargo test -p locus-core telemetry::usage_unknown_not_zero` |
| 16 | Assert Locus never computes a token count itself | 15 | `cargo test -p locus-core telemetry::never_counts_tokens` |
| 17 | A missing verb is absent, never synthesized empty | 1 | `cargo test -p locus-core telemetry::missing_verb_stays_missing` |
| 18 | Per-harness declared verb set drives what a test expects | 17 | `cargo test -p locus-core telemetry::expectations_per_harness` |
| 19 | `permission_request` raises an alarm | 1 | `cargo test -p locus-core telemetry::permission_request_alarms` |
| 20 | Two sources produce the same downstream shape for the same run | 4,8,9,11 | `cargo test -p locus-core telemetry::sources_indistinguishable` |
| 21 | Event-based test harness: "run this, assert these events" | 20 | `cargo test -p locus-core telemetry::event_assertions` |
| 22 | Stream normalized events to the UI over `Channel<Event>` | 20 | `pnpm -C apps/desktop test -- transcript/from-core` |
