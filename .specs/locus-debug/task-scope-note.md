# locus-debug task scope note

The core/CLI slice now has a production container-backed adapter bridge. `locusd` owns a shared
`ContainerRuntime`; `DockerContainerRuntime::launch_debug_adapter` starts the selected adapter with
Docker exec inside `locus-agent-<run_id>` and exposes its stdio through the bounded DAP transport.
The debug registry owns that process until `debug stop` or run cleanup. The recording process remains
available only as an explicit unit-test seam.

## Boundary

- Adapter IDs come from the run's host-written registration, which is derived from the effective
  project/role marketplace tool set. `DebugRunConfig` defaults to invoking the adapter tool by ID and
  permits argv additions only when the first executable remains that allowlisted adapter.
- `DebugRunConfig.command()` is passed as the generic DAP `launch.command` value. Adapter plugins
  interpret that value; core does not branch on a language or adapter name.
- If Docker is unavailable, `locusd` does not fall back to a recording adapter. `debug start` returns
  `debug adapter runtime unavailable: locusd has no container runtime`.

## Lifecycle and safety

- Docker exec stdio is bridged with a bounded output queue and backpressure; DAP frame limits remain
  enforced by `DapClient`.
- Termination sends DAP `disconnect` without waiting for a response, then closes exec stdin. The run
  container remains the final owner and kills any surviving exec descendants when the run ends.
- Adapter commands use argv, not a shell string. Configuration validation rejects a command whose
  executable is not the configured allowlisted adapter.

## Evidence

- All 19 exact task filters pass: focused `cargo test -p locus-core dap::...`, `cargo test -p locus-cli
  debug::...`, and `pnpm -C apps/desktop test -- editor/no-debug-ui`.
- `cargo clippy -p locus-core --lib -- -D warnings` passes.
- `cargo clippy -p locus-cli --bin locusd -- -D warnings` passes.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- A Docker end-to-end run was not available in this environment (`docker info` returned no server
  version); the production Bollard path is compiled and covered by the runtime-launch seam test.
