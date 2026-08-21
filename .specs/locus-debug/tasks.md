# locus-debug — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | DAP client crate: initialize, launch, the request/event loop | — | `cargo test -p locus-core dap::client` |
| 2 | Core-held session keyed by run id | 1 | `cargo test -p locus-core dap::session_in_core` |
| 3 | Adapter process long-lived in the agent's container | 2 | `cargo test -p locus-core dap::adapter_in_container` |
| 4 | Assert the CLI holds no state between invocations | 2 | `cargo test -p locus-cli debug::cli_is_stateless` |
| 5 | `locus debug start --config` resolving from project settings | 3 | `cargo test -p locus-cli debug::start` |
| 6 | Assert `--config` uses the same run command under an adapter | 5 | `cargo test -p locus-core dap::same_run_command` |
| 7 | `locus debug break FILE:LINE` with `--if` | 3 | `cargo test -p locus-cli debug::break` |
| 8 | `--log FMT` as a logpoint that prints and continues | 7 | `cargo test -p locus-cli debug::logpoint_continues` |
| 9 | Assert a breakpoint survives across CLI invocations | 7,4 | `cargo test -p locus-core dap::breakpoint_persists` |
| 10 | `run\|step\|next\|finish\|continue` | 3 | `cargo test -p locus-cli debug::stepping` |
| 11 | `stack`, `vars --frame N`, `eval EXPR` returning structured JSON | 3 | `cargo test -p locus-cli debug::inspection` |
| 12 | `locus debug stop` | 3 | `cargo test -p locus-cli debug::stop` |
| 13 | A paused program suppresses the idle guardrail | 7 | `cargo test -p locus-core dap::pause_suppresses_idle` |
| 14 | Five minutes at a breakpoint trips nothing | 13 | `cargo test -p locus-core dap::long_pause_no_trip` |
| 15 | Adapters as marketplace entries in the allowlist | 3 | `cargo test -p locus-core dap::adapters_are_tools` |
| 16 | Honest "not available" without an adapter | 15 | `cargo test -p locus-cli debug::honest_unavailable` |
| 17 | Adapter dies with the run, no orphan | 3 | `cargo test -p locus-core dap::adapter_dies_with_run` |
| 18 | Docs blob advising logpoints over breakpoints | 8 | `cargo test -p locus-core dap::docs_prefer_logpoints` |
| 19 | Assert no debug UI exists anywhere in the app | — | `pnpm -C apps/desktop test -- editor/no-debug-ui` |
