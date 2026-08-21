# agent-cli — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Socket client over `/run/locus.sock` with a request/response framing | — | `cargo test -p locus-cli sock::roundtrip` |
| 2 | Verb dispatch table, every verb a round trip | 1 | `cargo test -p locus-cli sock::all_verbs_are_round_trips` |
| 3 | Assert the CLI computes no domain answer locally | 2 | `cargo test -p locus-cli sock::no_local_logic` |
| 4 | Assert the CLI holds no state, cache or config | 1 | `cargo test -p locus-cli sock::stateless` |
| 5 | `--json` flag on every verb | 2 | `cargo test -p locus-cli json::flag_everywhere` |
| 6 | Compact output; assert no pretty-printing anywhere | 5 | `cargo test -p locus-cli json::never_pretty` |
| 7 | Key-packed encoding for uniform tables | 5 | `cargo test -p locus-cli json::key_packed` |
| 8 | Row threshold below which packing is skipped | 7 | `cargo test -p locus-cli json::threshold` |
| 9 | Measure packing at 50-60% smaller than minified on tabular data | 7 | `cargo test -p locus-cli json::packing_saving` |
| 10 | `locus ask` blocking and setting `waiting` | 1 | `cargo test -p locus-cli ask::blocks_and_waits` |
| 11 | `ask` reaches the human inbox with its session attached | 10 | `cargo test -p locus-core ask::reaches_inbox` |
| 12 | `locus run status` | 1 | `cargo test -p locus-cli run::status` |
| 13 | `locus run artifacts` | 1 | `cargo test -p locus-cli run::artifacts` |
| 14 | Assert `run` returns only this run's state | 12,13 | `cargo test -p locus-core run::own_state_only` |
| 15 | `locus agent invoke` starting a nested run in its own container and clone | 1 | `cargo test -p locus-core invoke::nested_run` |
| 16 | Invoke returns to its caller | 15 | `cargo test -p locus-core invoke::returns` |
| 17 | Depth limit 3 enforced in core | 15 | `cargo test -p locus-core invoke::depth_limit` |
| 18 | Fan-out limit 4 enforced in core | 15 | `cargo test -p locus-core invoke::fanout_limit` |
| 19 | Cycle check on the invoke graph | 15 | `cargo test -p locus-core invoke::cycle_check` |
| 20 | Assert all three limits are independent — none substitutes for another | 17,18,19 | `cargo test -p locus-core invoke::three_limits` |
| 21 | A workflow may lower the bounds and never raise them | 17,18 | `cargo test -p locus-core invoke::workflow_lowers_only` |
| 22 | `locus svc up\|down` on the project network | 1 | `cargo test -p locus-cli svc::up_down` |
| 23 | Assert no Docker socket is used by the agent for svc | 22 | `cargo test -p locus-core svc::no_docker_socket_for_agents` |
| 24 | A non-allowlisted verb fails with a clear message, not a socket error | 2 | `cargo test -p locus-cli sock::allowlist_message` |
