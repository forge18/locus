# screens-review

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · Views `telemetry`, `runs`, `artifact`

## Purpose

What happened, and was it any good. Three tabs — Telemetry, Runs, Artifacts. PLAN.md's split is that
**Dashboard is now and Review is after**: Status says whether the system is healthy at a glance, Review
is where you dig into a run that was not. Every number here is already a column, so these screens are
queries rather than new instrumentation.

## Governed by

- PLAN.md §Canonical event vocabulary — the action list these facets count
- PLAN.md §Artifacts — one viewer per kind; comments steer the agent that made it
- PLAN.md §Token discipline — cache rate and payload-by-tool as first-class metrics
- `docs/design_handoff_locus_desktop_ui/README.md` screens 7, 8, 9

## Contract

### Telemetry

Search bar with a mono-ish query, blinking caret, and the note "every event, every session · BM25 over
the normalized log". `.tag-outline` filter chips (`verify: failed`, `30d`) plus "Reset filters".

Four metric cards — Sessions, Events, **Tool errors** in `--bad` with a red hairline, Output tokens —
plus a 1.5fr sparkline card of 16 accent bars at 85% opacity.

A three-column 434px band:
- **Filters** — grouped facet chips on `--sf3` with counts in `--mu2`: harness, capture source, project,
  agent · role, model tier, verify (active chip = accent tint + accent inset ring), arbiter class,
  branch. **The branch group states the invariant**: `main 0` at `opacity:.5`, because Locus never works
  in `main` and the facet proving it is worth more than a sentence claiming it.
- **Actions** — the canonical event vocabulary as mono rows: 132px name, a 7px track on
  `rgba(238,242,246,.06)` with an accent (or `--bad`) fill, right-aligned count. Includes the
  `2 permission_requests` **alarm callout** — PLAN.md keeps that verb in the vocabulary precisely
  because one firing means a harness gate was left on and the run is about to hang — and the "missing
  verb is recorded as missing" note.
- **Tools** — the same row pattern at 112px labels, with an anomaly note.

Bottom: `SESSIONS (N)` table — When / Harness / Project · repo / Agent · role / Model(s) / Runs /
Events / Errors / Tokens / Status / Id. Numerics mono and right-aligned; status colored (accent
running, `--bad` stuck/aborted/handed-off, `--ok` closed, `--mu` waiting).

### Runs

Search ("a path, a tool name, an event verb"), a `.seg` control (Today / 7d / **30d**), counts, and
three right-aligned stats — spec-gap rate, noise reclassified, tokens per passing run. Then `RUNS (N)`:
When / Harness / Project · repo / Agent · role / **Model resolved** / Events / Errors / Tokens /
Verify / Id.

**Model resolved, not model tier.** PLAN.md records the actual model id on the run so spend and verify
pass rate are attributable to what really answered.

### Artifacts

Three panes. **222px list** — `REVIEW ARTIFACTS`, one entry per kind (diff, walkthrough, image with
OCR, recording with derived keyframes, diagram), then a dimmed `REFERENCE · NEVER IN THE INBOX` group
(finding, payload).

**That split is load-bearing.** Reference kinds are storage with a handle; without the split the inbox
fills with an agent's own scratch and the surface built to protect attention becomes the one that
spends it.

**Center** — header (accent kind tag, mono file name, locator, "one viewer per kind · three entry
points") over a unified diff: `@@` headers in `--mu2`, 26px gutter, the Develop tints, and the
commented line marked `inset 3px 0 0 var(--ac)`.

**306px right rail** — `COMMENTS STEER THE AGENT`: your comment on `--sf` with a 16px mono-initial
avatar, the agent's reply on `--sf2` + `--line2` ring, and a pulsing "run is still live · comment routed
into the session". Footer textarea with "Send to session" / "Resolve".

## Acceptance

1. Facet chips show counts, and an active chip carries the accent tint plus inset ring.
2. The branch facet shows `main 0` — the invariant is visible as data.
3. The Actions list contains exactly the twelve canonical verbs, no more.
4. `permission_request` renders as an alarm, visually distinct from an ordinary count row.
5. Runs shows the **resolved model id**, not a tier name.
6. Reference-kind artifacts appear only in the dimmed group and are labeled never-in-the-inbox.
7. The same artifact renders identically from all three entry points — one viewer per kind.
8. A commented diff line carries the accent left inset.

## Open

- Whether the Sessions and Runs tables need virtualization at 300 and 612 rows. `fixtures` measures it;
  this screen consumes the answer.
