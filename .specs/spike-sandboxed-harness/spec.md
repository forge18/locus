# spike-sandboxed-harness

**Milestone** M0 · **Depends on** none · **Blocks** `sandbox`, `run-supervisor`, `telemetry`

## Purpose

Answer the highest-risk unknown in the design: can a harness run inside a container, authenticated,
without that container ever holding a long-lived secret? If it cannot, the sandbox model changes and
several M1 features change with it. This is an experiment that produces a written finding, not a
feature that ships.

## Governed by

- PLAN.md §Credentials — the requirement, the host-proxy pattern, the lethal-trifecta threat model
- PLAN.md §Containers, §Images — two layers, one cache key, harness binary baked not host-installed
- PLAN.md §M0, Spike 1
- PLAN.md §Risks — "Risk — harness credentials"

## Contract

The spike delivers `spikes/01-sandboxed-harness/FINDINGS.md` answering four questions in writing:

1. **Auth.** Which mechanism authenticates a harness in a container without baking a secret into the
   image or mounting a long-lived credential file? Candidates to test, in order: a host proxy that
   injects the real credential into outbound calls so the container holds only a sentinel; a
   short-lived token minted per run over `/run/locus.sock`; an env var injected at container start.
2. **Image.** Does `locus/base-<harness>` build with the harness CLI present, and does `detect` fail
   the build when the binary is missing rather than silently producing a broken image?
3. **Events.** Does the run emit a stream that parses into the canonical vocabulary
   (PLAN.md:661-663), and does `usage` arrive with real numbers?
4. **Clone.** Does the harness work against `/workspace` as a container-local clone with no host mount?

Run against **two harnesses with different capture sources** — `claude` (hooks) and one of
`cursor` (ACP) or `antigravity` (stream-json) — because a mechanism that works for one capture path
proves less than it appears to.

## Acceptance

1. `spikes/01-sandboxed-harness/FINDINGS.md` exists and answers all four questions with a verdict, not
   a survey.
2. A container built from the spike's Dockerfile runs a real harness session end to end and the
   session's events are captured to a file.
3. `docker exec <container> env` and a filesystem scan show **no long-lived credential** — the finding
   states exactly what the container did hold and for how long.
4. The finding names **what would falsify the sandbox model** and what the fallback is if it did.
5. `dsh` and `hermes` are confirmed against real binaries or the finding records that they remain
   UNVERIFIED and why — PLAN.md:2248 calls this Spike 1's other half.

## Open

- Whether egress policy tiers belong at the same chokepoint as credential injection. PLAN.md says they
  do; the spike is where that is cheap to confirm or refute.
