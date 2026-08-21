# sandbox — tasks

**Spike 1 can void tasks 8-11.** If harness auth cannot be injected without a long-lived secret, the
credential mechanism changes and those four are rewritten against whatever the spike returned. The rest
of this file holds regardless.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `bollard` client, daemon connection, and lifecycle wrappers | — | `cargo test -p locus-core docker::connects` |
| 2 | `locus/base-<harness>` build per registered harness | 1 | `cargo test -p locus-core images::base_builds` |
| 3 | `detect` at build time; a missing binary fails the build | 2 | `cargo test -p locus-core images::detect_fails_build` |
| 4 | `locus/agent-<hash>` layer over the base with the tool set | 2 | `cargo test -p locus-core images::agent_layer` |
| 5 | Cache key over base digest, sorted tools, resolved pins | 4 | `cargo test -p locus-core images::cache_key` |
| 6 | Identical tool lists share one image | 5 | `cargo test -p locus-core images::shared_when_identical` |
| 7 | Editing a skill rebuilds no image | 5 | `cargo test -p locus-core images::config_is_not_a_layer` |
| 8 | Credential injection, in whatever shape Spike 1 settled | 1 | `cargo test -p locus-core creds::injects` |
| 9 | Scan a running container for a persisted credential | 8 | `cargo test -p locus-core creds::no_long_lived_secret` |
| 10 | Egress policy tiers applied at the chokepoint | 8 | `cargo test -p locus-core creds::egress_tiers` |
| 11 | One audit row per outbound call | 10 | `cargo test -p locus-core creds::outbound_audited` |
| 12 | Mount exactly `/run/locus.sock` rw and `/locus/config` ro | 1 | `cargo test -p locus-core container::two_mounts_only` |
| 13 | Assert no Docker socket reaches an agent container | 12 | `cargo test -p locus-core container::no_docker_socket` |
| 14 | `/workspace` as a container-local clone | 12 | `cargo test -p locus-core container::workspace_is_a_clone` |
| 15 | Assert the host working copy is unreachable from inside | 14 | `cargo test -p locus-core container::host_tree_unreachable` |
| 16 | `$LOCUS_PORT` allocator over 43000-43999, recorded on the run | 1 | `cargo test -p locus-core ports::allocates_unique` |
| 17 | Project network `locus-<project>`, joining agents and services | 1 | `cargo test -p locus-core net::project_network` |
| 18 | Assert cross-project containers are unreachable | 17 | `cargo test -p locus-core net::project_isolation` |
| 19 | `locus svc up\|down` starting a project service on that network | 17 | `cargo test -p locus-core svc::up_down` |
| 20 | PTY attached from the host to the container process | 12 | `cargo test -p locus-core container::pty_attaches` |
| 21 | Canary token written into the materialized config | 12 | `cargo test -p locus-core canary::present_in_config` |
| 22 | Canary detection on output; a deliberate leak is caught | 21 | `cargo test -p locus-core canary::detects_leak` |
| 23 | Per-run tool-call rate limit | 20 | `cargo test -p locus-core limits::tool_call_rate` |
| 24 | Boot reconciliation: alive container re-attaches, gone container closes as aborted | 1 | `cargo test -p locus-core container::reconciles_on_boot` |
