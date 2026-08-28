# sbx-runtime — tasks

**Spike 4 evidence can void tasks 4–5.** If `sbx template load` cannot import the image pipeline's
tar, those two are rewritten against sbx's actual import path and task 3's create invocation picks
the template that path produces. The rest of this file holds regardless. Rows marked *live* require
`SBX_LIVE=1` and a signed-in `sbx`; they skip honestly when the gate is absent.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Backend seam extracted from `sandbox` — create, prepare, attach ACP stdio, stop, remove, state; Docker becomes backend A, behavior unchanged | — | `cargo test -p locus-core sandbox::` |
| 2 | Machine-level `runtime = docker\|sbx` in core config, recorded on the run row, surfaced in `locus status --json` | 1 | `cargo test -p locus-core store::runtime_backend_recorded` |
| 3 | sbx lifecycle: create `locus-agent-<run_id>` with `-t`, per-run scratch, `-e`, `-p`; stop; `rm --force`; state from `sbx ls --json`; missing binary fails honestly (shim-tested) | 1 | `cargo test -p locus-core sandbox::sbx_lifecycle` |
| 4 | Template import: `docker save` both layers → `sbx template load` → `sbx create -t`; setup pre-pull; `detect`-at-build parity | 3 | `cargo test -p locus-core sandbox::sbx_template_import` |
| 5 | Template import proven against the real sandbox runtime | 4 | *live* `SBX_LIVE=1 cargo test -p locus-core sbx_live::template -- --ignored` |
| 6 | One-time `sbx policy init` detection at setup; an uninitialized machine errors naming the command | 3 | `cargo test -p locus-core sandbox::sbx_policy_gated` |
| 7 | Tier→allowlist mapping (`None`/`Model`/`Packages`/`Open`) to scoped post-create rules; relay, git-daemon, and service ports included per run | 3 | `cargo test -p locus-core sandbox::sbx_egress_tiers` |
| 8 | Egress audit reads response bodies, not exit codes (sbx denials exit 0); one row per outbound call | 7 | `cargo test -p locus-core sandbox::sbx_egress_audited` |
| 9 | TCP relay auth: `locus` CLI reaches locusd at `host.docker.internal` with the per-run nonce; the socket is never mounted; nonce reuse is rejected | 3 | `cargo test -p locus-core sandbox::sbx_relay_auth` |
| 10 | Workspace clone from the bare remote over `git://` through the allowed port; the host working copy is never mounted; `--clone` unused | 7, 9 | `cargo test -p locus-core sandbox::sbx_workspace_is_a_clone` |
| 11 | Push-back: `git push` over the same channel lands the run branch on the bare remote (`receive-pack`) | 10 | `cargo test -p locus-core sandbox::sbx_push_back` |
| 12 | Config through the scratch: fixed path, `LOCUS_CONFIG`, byte-identical to docker's `/locus/config`, canary present | 3 | `cargo test -p locus-core sandbox::sbx_config_materialized` |
| 13 | Credential parity: post-start scan finds sentinel + nonce only; `sbx secret` unused; no docker socket in the sandbox | 9, 12 | `cargo test -p locus-core sandbox::sbx_no_long_lived_secret` |
| 14 | ACP + artifacts: framed stdio over `exec -i` (1MB frames, hostile bytes, exit 7, EOF, clean stdout); artifacts cross the TCP relay with the docker CLI verbs — no container-copy path | 9 | `cargo test -p locus-core sandbox::sbx_acp_artifacts` |
| 15 | `$LOCUS_PORT`: two agents get distinct published ports, each reachable from a project container and unreachable off-machine; cross-project egress denied | 7 | `cargo test -p locus-core sandbox::sbx_ports_services` |
| 16 | Boot reconciliation + teardown: alive re-attaches, gone closes aborted, `rm --force`, scoped rules verified gone with the sandbox | 3, 7 | `cargo test -p locus-core sandbox::sbx_reconciles` |
| 17 | Live end-to-end: one real `pi` ACP session in an sbx sandbox on this machine — Spike 4's NOT-EXERCISED item, now exercised | 5, 10, 12, 14 | *live* `SBX_LIVE=1 cargo test -p locus-core sbx_live::acp_session -- --ignored` |
