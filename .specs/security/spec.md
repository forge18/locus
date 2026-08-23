# security

**Milestone** follow-on from the 2026-08-23 review · **Depends on** `sandbox`, `materializers`, `run-supervisor`, `store`

## Purpose

Harden the four findings from `.specs/security/REVIEW-2026-08-23.md`. Each was classified as either a
**code change** (a defect in code that exists) or a **design decision** (a control that was never built;
a decision was required before any implementation). All four are now decided; this spec converts those
decisions into buildable contracts.

## Governed by

- `.specs/security/REVIEW-2026-08-23.md` — the findings, the change-kind split, and the recorded decisions
- PLAN.md §Credentials, §Containers, §Sandboxing — the threat model and the ten defense layers
- The false-positive exclusions and confidence rubric in the `security-review` skill

## Decisions (locked 2026-08-23)

| Finding | Kind | Decision |
| --- | --- | --- |
| F1 | Design | **Per-project forwarding proxy** — packet-level egress enforcement on the existing project network, allowlisted destinations per run tier. MicroVM (**C**) rejected outright: not cross-platform (macOS/Windows via Docker). |
| F2 | Design | **Trusted-by-channel** — our extensions, the harness itself, or the user are the only instruction sources. Everything else is data-plane. User override ladder: **once / session / global**. No blocking gate. |
| F3 | Code (LOW) | **Boundary control** — route the raw provider error to the host's trusted log; forward a secret-free gist upstream; keep exact-match `redact()` as defense-in-depth. |
| F4 | Design | **Socket-boundary ownership check** — verify run→project/session ownership at `runtime/daemon.rs` before any ID-bearing verb routes. |

## Contract

**F1 — per-project forwarding proxy.** Egress enforcement moves from "the credential proxy gates one
proxied channel" to "all run egress passes a project-network forwarding proxy that applies per-run tier
filters." Tiers `None`/`Model`/`Packages`/`Open` map to allowlisted destination sets. A run on `Model`
cannot reach hosts outside its allowlist, regardless of the transport it attempts. Same-project isolation
(`none` shares no general network with egress-capable runs) is preserved.

**F1 topology — dual-network proxy sidecar.** Each project receives an internal agent network and a
Locus-managed forwarding-proxy sidecar attached both to that network and to a separate egress network.
Agent containers attach only to the internal network, so the proxy is their sole outbound peer. `None`
runs receive no proxy route; `Model` and `Packages` may use only their declared destinations; `Open`
permits general HTTP/HTTPS and `CONNECT` through the proxy for research, but never a direct socket.
The proxy records allow and deny decisions. It is a vendored Locus image built from this repository,
not a third-party runtime image. Its lifecycle, provider-derived model hosts, and package-registry
allowlist are part of this story rather than agent-image configuration. `Packages` starts with no
registry defaults: a project must explicitly declare each package host before a run can reach it.

**F2 — trust boundary, non-blocking.** Content authored by our extensions, the harness, or the user is
the instruction plane. All other content (workspace reads from the clone remote, fetched pages, other
agents' artifacts, generic tool results) is data-only. One standing operator rule in the always-on
context states this. The user overrides a mislabel in one interaction: **override once**, **override for
session**, or **override globally**. Nothing in the agent flow blocks by default.

**F3 — redaction by boundary, not enumeration.** Provider error bodies are written to the host's trusted
log; the `Err` propagated upstream carries only a secret-free gist (status + provider + reason category).
The existing exact-match `redact()` stays as defense-in-depth for the common case. No attempt to enumerate
URL-encoded/base64/truncated transforms.

**F4 — socket-boundary authorization.** `runtime/daemon.rs` verifies, for every ID-bearing verb, that
the authenticated run's project/session owns the target ID, **before** routing to any service. The store
methods are unchanged (they cannot know the caller). A regression proves run A cannot read run B's
artifacts or settings.
