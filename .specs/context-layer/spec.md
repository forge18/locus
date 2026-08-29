# context-layer

**Milestone** M3 · **Depends on** `memory`, `tool-compaction`, `materializers`, `telemetry` ·
**Blocks** nothing (R7's task extends `calibration-loop`'s acceptance in place)

## Purpose

Adopt the research-validated context-management increments decided in
[`CONTEXT-LAYER-FIT.md`](../../CONTEXT-LAYER-FIT.md) §9 (R1–R8). Every mechanism here is
already validated by published research or deployed-at-scale practice
([`RESEARCH-CONTEXT-MANAGEMENT.md`](../../RESEARCH-CONTEXT-MANAGEMENT.md) §13, §16.1) —
nothing in this spec is novel-mechanism research, and **no experiments are in scope**:
validation is deterministic assertions only, per the house `verify:` convention.

## Governed by

- PLAN.md §Memory — catalog, promotion, decay, recall (amended, not replaced)
- PLAN.md §Token discipline — prefix stability, mutable content last, cache rate as a column
- PLAN.md §Knowledge, as one model — the constitution stays outside the memory store
- CONTEXT-LAYER-FIT.md §9 — resolutions R1–R8 from the research fit discussion
- RESEARCH-CONTEXT-MANAGEMENT.md §13, §16.1 — the evidence-backed ordering this spec adopts

## Contract

### R1 — Eviction class, derived placement (amends memory)

A `eviction_class` column on memory records: `sticky` | `standard`, default `standard`.
Set at capture/promotion: records derived from unresolved errors and declared promotions
(R2) are `sticky`; everything else is `standard`. `disposable` is not a memory value —
tool output is already governed by the tool-compaction threshold. Catalog overflow never
drops `sticky` entries before `standard` ones, and drops `standard` by strength with the
existing log-line discipline.

A static derivation table in the materializer maps (knowledge-kind, lifetime,
`task_class`) → {placement zone, injection mechanism, eviction class}. The table is code,
written once, every harness; the assembled context order stays byte-deterministic. The
constitution and rules map to the always/never-dropped slot outside the memory store —
no change to their plane.

### R2 — Declared promotion (amends memory)

`locus memory promote`: skips the cluster-density check but still runs re-verification
and dedup, lands in the same probation metadata, and decays if never recalled. `--json`
out, compact. The density path is unchanged.

### R4 — Research-class diversity dedup (amends memory)

When selection depth k > 1 (`research` class only), suppress candidates whose embedding
similarity to an already-selected candidate exceeds the threshold (MMR-lite). `code` and
`plan` (k=1) are unchanged. Selection-time, ranker-local, no new objects.

### R5 — Freshness at the tail, freeze at the head (amends memory + token discipline)

The frozen `SessionStart` catalog snapshot is unchanged (PLAN: "the memory catalog is a
snapshot"). A second, **append-only "new since snapshot" section** is injected at the
mutable tail: paths + one-liners for memories captured during the run, append-ordered,
merged into the next session's snapshot. Overflow of the tail section logs drops; it
never rewrites earlier tail entries. The head of the assembled context is byte-identical
whether or not the tail section exists.

The tail budget is **derived, not fixed**: computed from the same effective-window
derivation as the catalog cap (PLAN derives 800 tokens ≈ 40 entries from effective
context; the tail takes a stated fraction of the mutable zone), so it scales with the
model the run is bound to. Appends are capacity-aware per ContextBudget (arXiv 2604.01664):
before incorporating a new tail entry, check remaining capacity and compress or drop tail
entries first — the plain rule, not a learned policy. This is the validated core of the
budget-aware result (which beat the static "summarize only when full" baseline), adopted
at the tail only; the frozen head and its cache arithmetic are untouched.

### R6 — Recitation block (new, run supervisor)

On task-state changes the run supervisor emits a one-to-three-line recitation block
through the existing hook injection path: current objective, current step, unresolved
error count. Tail placement, next to the other mutable content. Absent when no plan is
active. It never mutates the frozen head, never calls a model, and inherits the 100ms /
exit-0 discipline.

### R7 — Cache-rate acceptance in calibration-loop (amends calibration-loop)

Context-policy changes are regressions. The calibration loop gains a paired-run
comparison support and a **cache-rate non-regression** acceptance criterion computed from
`usage.cache_read` / `usage.input`, which already exist on every event.

### R8 — Context attribution as a view (new, telemetry)

A `context_attribution` SQL view over existing events: injection and recall events ↔
verify outcomes ↔ `tool_result` rows, joined with the per-run materialization snapshot
that already identifies base-context/rules/skills content. Adds a **verification-cost**
pair of columns (the verify command's own duration and tokens). View only — no backfill,
no new instrumentation, no correlation id unless the view later fails to disambiguate
mid-run rule loads.

## Non-goals

- No MCP anything (repo invariant); the tool surface stays first-party CLI, so policy
  binding stays coarse: container + git/branch model + credential-proxy audits.
- No model calls inside hooks; no per-turn free rebuild; no unfreezing of the catalog
  snapshot (freshness is R5's tail section, not a live view).
- No learned budget policy: the ContextBudget capacity check (arXiv 2604.01664) is
  adopted as a plain rule at the tail only; the paper's learned sequential-decision
  policy is research-only. No offline consolidation job (G5), no typed-context
  validation pass (G6) — deferred, not decided.
- No experiments: learned assembly, active-inference acquisition, and decay-at-boundary
  measurement are research-only (RESEARCH-CONTEXT-MANAGEMENT.md §13.6, §16.1) and out of
  adoption scope.
- Nothing in core names a harness; every harness entry stays complete; assembled output
  never begins with `{` (memory task 11's rule extends to the new tail section).

## Acceptance

1. `eviction_class` folds from the log per the event-sourcing carve-out rules; default
   `standard`; no stored value contradicts its capture origin.
2. Catalog overflow drops `standard` by strength, never `sticky`, and logs every drop.
3. The derivation table is pure: same inputs, byte-identical assembled order, per run and
   across runs.
4. `locus memory promote` promotes without density, refuses nothing silently, and its
   records decay exactly like any other.
5. Within a run, a memory captured mid-run appears in the tail section and not in the
   frozen head; head bytes are identical with and without tail entries.
6. The recitation block is ≤3 lines, tail-placed, absent without a plan, and its updates
   never touch the frozen head.
7. Research-class selection contains no pair above the similarity threshold; code/plan
   selection is byte-identical to today's.
8. The calibration loop computes cache-rate per arm and flags non-regression violations.
9. `context_attribution` answers "what was in front of the agent when it failed" for a
   fixture run, including verify cost, with no new event shapes.

## Open

- The similarity threshold for R4 dedup; starts conservative (suppress only near-
  identical) and is a setting, not code. The mechanism's provenance is MMR-style
  diversity reranking (Carbonell & Goldstein, SIGIR 1998 — background canon, not
  retrieved this session); no validated constant exists in the corpus, so none is
  invented here.

Resolved 2026-08-29: the tail budget was Open, then closed as derived — see R5. It is
no longer a setting: it follows from the effective-window derivation plus the
ContextBudget capacity check, which is why it scales per model.
