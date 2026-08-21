# sandbox

**Milestone** M1 · **Depends on** `spike-sandboxed-harness`, `harness-registry` · **Blocks** `run-supervisor`

## Purpose

One container per agent run, and the credential handling that makes it safe. This is where the design's
isolation claim is either true or merely asserted.

**Spike 1 settles the credential mechanism**, and this spec's contract is written to whatever it
returns — the acceptance criteria below are mechanism-independent on purpose, because they state the
property that must hold rather than the technique that achieves it.

## Governed by

- PLAN.md §Containers, §Images — two layers, one cache key
- PLAN.md §Credentials — the requirement, the lethal trifecta, the ten defense layers
- PLAN.md §The git model — clone, not mount
- PLAN.md §Permissions — enforcement is the container, not the harness

## Contract

**Three container kinds:**

| Container | Lifetime |
| --- | --- |
| `locus-postgres` | per machine |
| `locus-agent-<run_id>` | per agent run — one harness process, one session, no TUI, PTY from the host |
| `locus-svc-<project>-<name>` | per project — the project's own Postgres, Redis, browser |

Network `locus-<project>` joins a project's agents and services. Agents reach each other and the
project's services; they do not reach other projects.

**Two image layers, one cache key:**

| Layer | Rebuilt when |
| --- | --- |
| `locus/base-<harness>` | the harness version pins change |
| `locus/agent-<hash>` | the agent's tool list or a tool's pin changes |

`<hash>` is over (base image digest, sorted tools list, resolved marketplace pins). **Config is never a
layer** — it is materialized per run, so editing a skill invalidates no image.

**The harness binary lives in the image, never on the host.** `detect` runs at image build and its job
is to fail the build when the binary is missing, not to find one you already have.

**Mounts** — and only these two:

| Path | Mode | Contents |
| --- | --- | --- |
| `/run/locus.sock` | rw | the host daemon socket |
| `/locus/config` | ro | this run's materialized config |

**`/workspace` is a clone, not a mount.** An agent cannot touch your working copy because it does not
have it. A bind-mounted worktree can always be escaped by a path bug; a filesystem that was never
mounted cannot.

**`$LOCUS_PORT` is allocated by the core** from 43000-43999 and recorded on the run. Two agents on one
project otherwise collide on whatever the repo's dev server defaults to.

**Agents get no Docker socket.** A container needing a service asks the core: `locus svc up postgres`.
This avoids Docker-in-Docker and keeps the root-equivalent daemon socket away from agents.

**Four defense layers that need naming so they are not skipped**, beyond the sandbox itself:
rate-limiting tool calls per run (injection usually needs volume); a **canary token** in the
materialized config that must never appear in output; anomaly detection as a query over the normalized
tool sequence; and gates calibrated to what is irreversible, touches production, or reaches credentials.

**Stated honestly:** containers share the host kernel. This is weaker isolation than a microVM, and the
boundary is a kernel boundary.

## Acceptance

1. A run's container holds **no long-lived credential** — asserted by scanning env and filesystem after
   start, whatever mechanism Spike 1 chose.
2. `/workspace` is a clone; the host's working copy is unreachable from inside, proven by attempting it.
3. Only the two documented mounts exist; a third fails the test.
4. No Docker socket is present in an agent container.
5. Two agents on one project get different `$LOCUS_PORT` values, both recorded on their run rows.
6. Two agents with identical tool lists share one `locus/agent-<hash>` image.
7. Editing a skill rebuilds **no** image.
8. `detect` failing at build produces a failed build, not a broken image.
9. The canary token appears in the materialized config and never in captured output; a test that leaks
   it deliberately is detected.
10. An agent on project A cannot reach a container on project B's network.

## Open

- Egress policy tiers. PLAN.md puts them at the same chokepoint as credential injection; whether that
  holds depends on Spike 1's mechanism, so the tier names and their defaults are settled with it.
