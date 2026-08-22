# memory

**Milestone** M3 · **Depends on** `store`, `event-store`, `telemetry`, `materializers` · **Blocks** `calibration-loop`

## Purpose

What an agent recalls: scoped facts with provenance, embeddings and decay. Written once in Rust and
reaching every harness through the same CLI, so **a harness swapped mid-project keeps its memory** —
nothing is stored in a harness's own format.

The scoping decision matters more than the mechanism. Three of the four kinds of knowledge are **not**
memory, and keeping them out is what keeps the store small enough to be trustworthy.

## Governed by

- PLAN.md §Memory — the four layers, capture, injection, promotion, decay, recall, the keeper
- PLAN.md §Knowledge, as one model — why three kinds are not memory
- PLAN.md §Token discipline — the frozen catalog and prefix stability
- PLAN.md §Event sourcing and its two carve-outs — memory holds both of them

## Contract

**Four layers by lifetime.** Locus owns three; working memory belongs to the harness.

| Layer | Lives in | Dies when | Shared |
| --- | --- | --- | --- |
| Working | the context window | the run ends | no |
| Short-term | probation buffer | promoted, or aged out | no |
| Long-term | Postgres | decays below threshold | yes, by role |
| Written | git — artifacts | never | everyone |

**There is no shared short-term.** Two agents sharing scratch state is Hermes's documented failure.
Sharing begins at long-term, after consolidation, so the promotion boundary and the trust boundary are
the same line.

**Probation is project-scoped, not session-scoped.** Cluster density is cross-session by nature: one
session yields one observation, and the pattern only appears once three sessions have yielded three.

**Capture is hooks, not tools.** Tools are agent-initiated, so the model must *remember to remember*.
**Hooks log and inject; they never think** — a hook cannot reuse the agent's LLM and fires on every tool
call, so the injection path carries a **100ms timeout** and the logging path **never touches the socket
synchronously**.

**Injection is a catalog, not content.** `SessionStart` emits paths and one-line summaries, capped at
**800 tokens ≈ 40 entries**; bodies arrive through `locus memory recall`. The cap is derived: effective
context is far below the marketed window, static anchors should hold under 10-15% of it, and 40 entries
is comfortably above what real projects occupy. **Exceeding 40 is a consolidation trigger, not an
eviction.** Output must not begin with `{`.

**The catalog is a snapshot, frozen at `SessionStart`** — for prefix-cache preservation as much as
memory hygiene.

**Promotion: three checks.** Re-verification (against `codanna`, a test run, or that run's verify
result); deduplication by path addressing with subject-aware embedding first; and importance
**measured, not predicted** — a memory recalled into a passing run is important. Event-driven on
cluster density: candidates wait until **three** target the same path. Originals are archived, never
deleted; on overflow, promote by score and drop the rest **with a log line**.

**Decay: two drivers, selected by content.** Memories naming a code symbol invalidate on *change* —
signature changed invalidates, body changed flags for re-verification, AST unchanged means nothing
happened. Everything else ages on an Ebbinghaus curve where `active_days` counts only days the project
saw a run. Half-lives: strategy 38d, fact 24d, assumption 19d, failure 11d.

**Chain-aware pruning**, which is what stops this sinking: a rare critical instruction is exactly the
low-frequency high-importance detail that vanishes by the third compression pass, because frequency and
importance are anticorrelated for the memories that matter most. **A decayed memory survives if any
graph neighbour is still strong.**

**Cold-start guard:** a memory is not eligible for pruning until it has survived one keeper pass in
which a query could plausibly have matched it. Without this, a new store prunes its own seed corpus.

**Recall depth reverses by task class.** On factual QA, accuracy rises with *k*. On agentic tasks the
sign flips — success falls from 32.1% at k=1 to ~25% at k=5, and flat retrievers drop *below* the
no-memory baseline. Over-retrieved agents do not simply fail, they wander.

| Task class | Substrate | k | Graph expansion |
| --- | --- | --- | --- |
| code | flat, similarity only | 1 | off |
| plan | distilled procedural cues | 1 | off |
| research | structural, hierarchical | high | on |

**Turn-level body injection is off for `code` and `plan`** — a missed injection costs one tool call; a
wrong one measured below the no-memory baseline.

**The keeper is an ordinary agent definition**, running at `high` tier, triggered on genuine project
idle. **The primary agent has no memory-edit tools at all** — memory management is the keeper's job
exclusively.

**Memory holds both carve-outs, and they are declared here.** Facts, scope and provenance fold from
the log like everything else. Two columns do not, and carry a `carve_out` annotation:

| Column | Why it cannot fold |
| --- | --- |
| `embedding` | a model output, not a function of the events behind its text, and not reproducible across embedding-model versions |
| `confidence` / decay state | a function of wall-clock time — the Ebbinghaus curve over `active_days` — not of appended entries |

Decay is therefore **evaluated at read** from `last_active` plus the curve, rather than materialized and
folded. The alternative is writing tick entries so the log can model a clock, which is the point at
which event sourcing stops paying for itself.

**`locus rebuild` restores memory's text and loses its vectors.** That is not a bug in the rebuild; the
vectors were never derived from the log. Recovery is a restore plus a re-embed, and it is one of the
two reasons backup is non-deferrable.

**An empty store degrades cleanly**: no catalog, no injection, no recall — agents run exactly as they
would with no memory layer, which is the correct baseline.

## Acceptance

1. A memory written by an agent on one harness is recalled by an agent on a **different** harness.
2. The `SessionStart` catalog never exceeds 800 tokens, and exceeding 40 entries triggers consolidation
   rather than eviction.
3. The catalog is identical for the whole run even when the store changes underneath it.
4. Catalog output never begins with `{`.
5. The injection hook returns within 100ms or emits nothing; the logging hook never blocks on the socket.
6. Promotion fires at the third candidate on a path, not the first or second.
7. A signature change invalidates a symbol memory; a formatting commit changes nothing.
8. A decayed memory with a strong graph neighbour survives pruning.
9. A memory that has never had a chance to match is not pruned.
10. `code`-class recall returns k=1 with graph expansion off; `research` returns high-k with it on.
11. The primary agent cannot write memory directly unless `write: direct` is set.
12. An empty store produces no catalog and no recall, and runs succeed anyway.
13. `embedding` and decay state carry `carve_out`; a new non-foldable column without one fails the
    schema test.
14. Decay is computed at read from `last_active` and the curve — no tick entry is ever written to the
    log.
15. `locus rebuild --schema memory` restores facts, scope and provenance and leaves stored vectors
    byte-identical, recomputing none of them.

## Open

- PLAN.md gives the decay formula and half-lives but not the initial `importance` assignment. Measured
  importance needs a seed value before anything has been recalled.
