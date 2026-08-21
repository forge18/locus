# Spike 1 — sandboxed harness · QUESTION

Written before the experiment. Nothing below is a conclusion.

**Governed by** PLAN.md §Credentials, §Containers, §Images, §M0 Spike 1, §Risks — "Risk — harness
credentials". Contract in `.specs/spike-sandboxed-harness/spec.md`.

## The unknown

PLAN.md §Credentials states a *requirement, not a mechanism*: getting a harness authenticated inside a
container must be easy to use and secure, and **the container should not hold a long-lived secret**.
It names Docker's host-proxy pattern as one way to satisfy that and defers the choice to this spike.

Every M1 feature that touches a container inherits the answer. `sandbox` cannot state its mount set or
its egress story until the injection chokepoint is fixed; `run-supervisor` cannot say what it hands a
container at start; `telemetry` cannot claim the four capture paths are interchangeable until at least
two of them have been observed through the same boundary.

## The four questions

### Q1 — Auth

Which mechanism authenticates a harness in a container without baking a secret into the image or
mounting a long-lived credential file?

Three candidates, tested in this order, each judged on the same four axes — **what the container
holds**, **for how long**, **what a compromised container can do with it**, and **what it costs to set
up per project**:

| | Candidate | What the container gets |
| --- | --- | --- |
| **A** | Host credential proxy | a sentinel string; the host swaps in the real credential on the way out |
| **B** | Per-run token minted over `/run/locus.sock` | nothing at start; a revocable per-run token on request |
| **C** | Env var injected at container start | the real credential, for the life of the run |

**C is the baseline to beat, not a straw man.** It is what most of the field does, it is trivial to
operate, and if A and B cannot beat it on exposure without costing more than they save, the honest
finding says so.

### Q2 — Image

Does `locus/base-claude` build with the harness CLI present, and does `detect` **fail the build** when
the binary is missing rather than silently producing an image whose breakage surfaces at first run?

PLAN.md §Images puts `detect` at image-build time precisely so it cannot find a binary that happens to
be on the host. The failing case is the one that has to be proven.

### Q3 — Events

Does a run inside the container emit a stream that parses into the canonical vocabulary
(PLAN.md §Materializers, "Canonical event vocabulary"):

```
session_start  user  assistant  thinking  tool_call  tool_result  tool_error
permission_request  subagent_start  subagent_stop  aborted  session_end
```

and does `usage { input, output, cache_read, cache_write }` arrive with **real numbers**, not nulls?

Run against **two harnesses with different capture sources**, because a mechanism proven on one
capture path proves less than it appears to.

### Q4 — Clone

Does the harness work against `/workspace` as a **container-local clone** — cloned from a bare local
remote on the host, with no bind mount of any working copy?

## Harnesses under test

| Harness | Capture source | Why this one |
| --- | --- | --- |
| `claude` | `hooks` | the reference harness; every extension native, richest capture path |
| second | not `hooks` | a second capture source, so the credential boundary is not proven against one dialect only |

The second harness is named in FINDINGS.md along with why it was the one available. `.specs` names
`cursor` (ACP) or `antigravity` (stream-json) as the preferred pair.

`dsh` and `hermes` are the other half of this spike (PLAN.md §Harness registry): both are UNVERIFIED
against running binaries. Each ends this spike marked VERIFIED or UNVERIFIED-with-a-reason.

## What falsifies the sandbox model

Stated now so the result cannot be graded after the fact. **Any one of these sends the container model
back for redesign:**

1. **No mechanism keeps a long-lived credential out of the container.** If every candidate ends with
   the real key readable inside the container for the life of the run, then "the container should not
   hold a long-lived secret" is not a property of the container — it has to become a property of
   something above it (a microVM, or a per-run credential issued by the provider), and PLAN.md
   §Credentials' closing paragraph about kernel-vs-hypervisor boundaries stops being a footnote.
2. **The harness cannot be redirected.** If the harness ignores its base-URL/endpoint configuration, or
   pins certificates, a host proxy cannot sit in the path — which takes egress policy tiers and the
   per-call audit row down with it, since PLAN.md puts them at the same chokepoint.
3. **`detect` cannot fail a build.** If a base image can be produced without its harness binary, the
   image layer stops being a reproducibility guarantee and every run inherits a host-install problem.
4. **The clone model does not hold.** If a harness needs a host filesystem path it was not given, then
   `/workspace` as a container-local clone fails and the isolation argument in §The git model — "an
   agent cannot touch your working copy, because it does not have it" — is no longer true.
5. **`usage` is unavailable or fabricated.** If real token numbers cannot be captured per run, spend
   tracking, the cache-rate alert, and the guardrail budgets are all reading a number Locus invented.
   PLAN.md is explicit that Locus never counts tokens itself.

## The fallback if it is falsified

Named now, so the finding is a decision rather than a report:

- **1 falsified** → credential injection moves out of the container boundary. Either the harness runs
  on the host under a per-run user with the container holding only the workspace, or the isolation
  boundary becomes a microVM and the "Linux and macOS, no proprietary dependency" property in
  §Credentials is paid for.
- **2 falsified** → the proxy becomes a CONNECT-level egress gateway with policy and audit but **no**
  credential injection, and Q1 falls back to candidate C with the exposure window recorded as accepted
  risk rather than solved.
- **3 falsified** → `detect` moves to a post-build smoke run of the image, and `ci`'s materialization
  smoke test grows a per-harness image check.
- **4 falsified** → the affected harness is marked as requiring a mount, and `sandbox` gains a
  per-harness mount exception — which is a real weakening of the isolation claim and must be recorded
  as one, not absorbed silently.
- **5 falsified** → `usage` is null for that harness and spend reads *unknown*, per PLAN.md's "a
  missing verb is recorded as missing, never synthesized".

## What this spike does not decide

- The egress policy tier names and their defaults. The spike says only whether policy **can** live at
  the same chokepoint as injection; the tiers themselves are settled in `sandbox`.
- Image size, build time, or layer caching strategy beyond "it builds".
- Whether the second harness ships at M1.
