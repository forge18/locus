# sbx-runtime

**Milestone** M1.6 · **Depends on** `sandbox`, `run-supervisor`, `security` · **Blocks** —

## Purpose

A second runtime backend for the agent container: Docker's `sbx` (Docker Sandboxes) behind the
contract `.specs/sandbox` already states. Three reasons, in weight order:

- **Isolation.** PLAN.md §Containers states honestly that Docker containers share the host kernel —
  weaker than a microVM. An sbx sandbox is a per-run VM. The agent boundary gets the stronger story
  without changing the contract.
- **Platform.** This repo's Docker-backed verification is blocked while Colima is unavailable, and
  sbx ran on this same machine without it: cached create 3.2s, resume from stop 1.1s, framed stdio
  7/7, git clone through the egress proxy. The evidence is
  [Spike 4](../../spikes/04-sbx-runtime-verify/FINDINGS.md).
- **Egress.** sbx ships default-deny network policy with sandbox-scoped allows — the property
  security F1 requires, enforced by a chokepoint that already exists instead of a second proxy to
  run.

Docker stays the default. `sbx` is opt-in per machine. Every mechanism below traces to Spike 4
evidence, existing code, or an sbx CLI flag; nothing in this spec is an open question.

## Governed by

- PLAN.md §Containers, §Credentials, §The git model — the properties preserved unchanged: one
  harness process per run, no TUI, ACP over stdio, clone-not-mount, no long-lived credential
- [.specs/sandbox/spec.md](../sandbox/spec.md) — the runtime-independent contract this implements a
  second backend for; its acceptance criteria are the ones this spec re-proves on sbx
- [.specs/security/spec.md](../security/spec.md) F1 — the egress tier contract. F1 rejected a
  microVM option as not cross-platform *before sbx existed*; the Docker-path decision is untouched,
  and this spec records the premise change instead of silently contradicting it
- [Spike 4 findings](../../spikes/04-sbx-runtime-verify/FINDINGS.md) — the measured basis for every
  transport, policy, and lifecycle claim

## Contract

**A runtime seam, not a second supervisor.** `sandbox` gains a backend interface — create, prepare,
attach ACP stdio, stop, remove, state — with today's Docker path as the first implementation and
`sbx` as the second. Backend selection is machine-level core config (`runtime = docker | sbx`,
default `docker`), recorded on the run row and constant for the run's life. A configured backend
missing at run start fails the run with an actionable message; no silent fallback — the same honesty
rule `locus-debug` set.

**Agents only.** `locus-postgres` and `locus-svc-<project>-<name>` stay on Docker: they are shared
and long-lived, and they network to each other on `locus-<project>`, which sbx does not model (its
verified surface is sandbox-to-host only). An sbx-backed agent reaches project services at
`host.docker.internal:<port>` through a scoped allow; `locus svc` publishes the port when the
project's runtime is `sbx`.

**Images carry over unchanged.** The two-layer model (`locus/base-<harness>`, `locus/agent-<hash>`)
builds exactly as today. `sbx template load` imports the built image tar into the sandbox runtime;
`sbx create -t` selects it. Cache keys, detect-at-build, and config-is-not-a-layer are
backend-independent and already tested. Setup pre-pulls, because Spike 4 measured the first create
at 37s (template pull) against 3.2s cached.

**One mount, and it is a scratch — the workspace stays a clone.** sbx has no arbitrary-mount flag;
its bind-mount preserves the host path inside the sandbox (observed in Spike 4). The run mounts
exactly one host dir: an empty per-run scratch. Under it, `workspace/` is the agent's working dir
and `.locus/config/` holds the materialized config — byte-identical to docker's `/locus/config`
content, canary included — located by `LOCUS_CONFIG`. The in-sandbox `locus` entrypoint clones
`/workspace` from the per-project bare remote, served by `git daemon --enable=receive-pack` on a
loopback port the run's policy allows; push-back is `git push` over the same channel. Clone and
raw-TCP git through the proxy are proven in Spike 4. sbx's own `--clone` mode is **unused**: it
clones the host working repo, and uncommitted working-copy state is exactly what the git model
keeps away from agents.

**The socket becomes TCP; the nonce carries the boundary.** `/run/locus.sock` is not mounted. The
agent's `locus` CLI reaches locusd at `host.docker.internal:<relay port>` with the per-run nonce —
the authenticator the M0 socket constraint already requires on macOS. The relay port and the
git-daemon port are the run's first two scoped allows.

**Egress: the same tiers at sbx's chokepoint.** Security F1's tier sets
(`None`/`Model`/`Packages`/`Open`) map to scoped per-sandbox allow rules applied immediately after
create (Spike 4: scoped allows require the sandbox to exist). `sbx policy init` is a one-time
setup step; an uninitialized machine errors naming the command. Local `--deny-network` may narrow
but never widen. **Denials surface as HTTP 4xx with exit code 0** — the egress audit reads response
bodies, not exit codes, and records one row per outbound call, same schema as docker runs.

**Locus uses sbx as a runtime, nothing more.** `sbx mcp`, `sbx skills`, `sbx secret`, `sbx kit`,
and `.sbxenv.yaml` stay unused. Config reaches the sandbox only through the materialized scratch
and `-e` (sentinel, relay base URL, `LOCUS_PORT`); secrets never enter `sbx secret`; MCP stays
forbidden.

**Lifecycle.** Sandbox name `locus-agent-<run_id>`; `sbx ls --json` is the state source; boot
reconciliation re-attaches to an alive sandbox and closes a gone one as aborted; teardown is
`rm --force` plus an assertion that the sandbox's scoped rules died with it (Spike 4: they do).

**Backend-independent defenses are not re-implemented.** Canary leak detection, tool-call rate
limiting, anomaly queries, and gates run at the daemon and relay layer and apply to sbx runs
unchanged; the post-start credential scan asserts sentinel-plus-nonce only.

## Acceptance

1. A run with `runtime = sbx` records `sbx` on its run row and never calls the Docker API; a docker
   run never shells out to `sbx`.
2. ACP over `sbx exec -i` round-trips framed JSON-RPC byte-exact — 1MB frames and CRLF/NUL/high-byte
   payloads survive, exit code 7 propagates, stdin EOF closes the session — and the
   `Sandbox … started successfully` line never appears on stdout.
3. `/workspace` holds a clone of the bare remote; the host working copy is unreachable from the
   sandbox, and `--clone` is absent from the create invocation.
4. Push-back lands the run branch on the bare remote; host-side readers see it.
5. Exactly one host dir is mounted — the per-run scratch — and the materialized config inside it is
   byte-identical to docker's `/locus/config` and carries the canary.
6. The run tier's allowlist is the sandbox's only egress; an unallowed destination is blocked and
   recorded even though the client's exit code is 0.
7. One audit row per outbound call, identical schema to docker runs.
8. The post-start scan finds the sentinel and nonce only; `sbx secret` is unused; no docker socket
   is present in the sandbox.
9. Two agents on one project get different `$LOCUS_PORT` values, both published and recorded, each
   reachable from a project container and from nothing outside the machine.
10. A missing `sbx` binary or an uninitialized policy fails the run with an actionable message,
    never a fallback.
11. Boot reconciliation: an alive sandbox re-attaches; a removed sandbox closes as aborted.
12. The base template's `detect` fails the build when the harness binary is missing — parity with
    the docker path.

## Open

None. The decisions this feature needed — credential path, egress chokepoint, template mechanism,
default backend — are made above. One follow-through rides the merge: PLAN.md §Containers still
describes a single runtime and gains the backend paragraph (interface, docker default, sbx opt-in)
when this lands. That is a doc change, not a new decision.
