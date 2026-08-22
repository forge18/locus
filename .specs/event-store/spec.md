# event-store

**Milestone** M1 · **Depends on** `store` · **Blocks** `board`, `workflow-engine`, `mail`, `memory`, `wiki`, `planning-module`, `guardrails`

## Purpose

Locus's own state is an append-only log, and every queryable table is a fold over it. The log is the
only thing written; a row is a cached answer, never a fact.

This is a deliberate choice made when nothing was built, against two live objections — see PLAN.md
§Event sourcing and its two carve-outs for why it was taken anyway and what it costs.

## Governed by

- PLAN.md §Event sourcing and its two carve-outs — the decision, the line, and the price
- PLAN.md §Data model — the eight schemas, plus `log`
- PLAN.md §Canonical event vocabulary — why the harness transcript is a *separate* log

## Contract

**Two logs, one ordering.**

| Log | Holds | Vocabulary | Written by |
| --- | --- | --- | --- |
| `agents.events` | the harness transcript | the twelve verbs, closed | telemetry normalizers |
| `log.entries` | Locus's domain events | open, versioned per kind | every core service |

They stay separate because the telemetry vocabulary is **closed and enforced at the type level**
(`telemetry` acceptance 2) — a `task_moved` entry in that enum would destroy the property that makes
harness output testable. They share one `stream_pos` counter per project, so *"everything since N"*
spans both and a consumer never has to merge two orderings by timestamp.

**A domain entry:**
```
log.entries
  stream_pos      BIGINT, monotonic per project, assigned by the core
  project_id
  kind            'task.moved', 'workflow.iteration_recorded', 'mail.sent', …
  v               SMALLINT — the version of THIS kind's payload
  payload         JSONB
  actor           run_id | 'human' | 'system'
  caused_by       nullable stream_pos — the entry that led to this one
```

**The fold is synchronous and in the same transaction as the append.** Appending an entry and updating
every projection it touches is one transaction, so **a projection is never stale and there is no
caught-up question to answer**. This is only available because the core is the sole writer — the same
property that made a Postgres sequence the wrong choice for `stream_pos`.

**Telemetry is exempt from synchronous projection.** `agents.events` is appended raw and its aggregates
— runs-by-hour, cost per session, verify rates — are computed by query against stored events, not
maintained as projections. Telemetry is the hot path: every tool call on every run writes to it, and
putting projection work inside that transaction would tax the highest-volume writer in the system to
serve dashboards that tolerate a query.

### The two carve-outs

The rule is not "these two schemas are exempt." It is one line, and it happens to cut through two
schemas:

> **The fold produces everything except what a model or a clock produced.**

| Carve-out | Where | Why it cannot fold | What is authoritative |
| --- | --- | --- | --- |
| **Embeddings** | `memory.store`, `wiki.pages` | an embedding is a model output, not a function of the events that produced its text; it is not reproducible across embedding-model versions | the stored vector |
| **Decay and confidence** | `memory.store` | a function of wall-clock time, not of appended events; folding it means either re-deriving on every read or writing tick entries so the log can model a clock | `last_active` plus the curve, evaluated at read |

Both are **declared, not discovered**: a `carve_out` annotation on the column, and a test that fails if
a new non-foldable column appears without one. The facts and pages themselves still fold — only these
columns sit outside.

**Rebuild does not touch a carve-out.** `locus rebuild` replays the log into the foldable projections
and **leaves embeddings and decay state exactly as they are**, because they were never derived from the
log and there is nothing to re-derive them from. A rebuild after volume loss therefore restores the
board, workflows and mail in full, and restores memory and wiki *text* in full with their vectors
missing — which is a re-embed, not a replay, and is why backup stays non-deferrable.

```
locus rebuild [--schema <name>] [--to <stream_pos>] [--into <scratch-db>]
```

`--to` is what makes time-travel free where it is free: the board and workflows as of any point.

**Versioning is forever.** Every `(kind, v)` ever written must stay foldable. A fold that meets an
unknown `(kind, v)` **refuses and halts** rather than skipping the entry — a skipped entry produces a
projection that is quietly wrong, which is the failure mode event sourcing exists to prevent.

**This is the obligation the design takes on knowingly.** Locus already carries permanent
schema-evolution cost against twelve third-party harness formats it does not control; this adds a
second, for events it does own. Owning them is the difference — a `task.moved` payload changes when
Locus changes it, on Locus's schedule.

## Acceptance

1. `log.entries` exists with `stream_pos`, `kind`, `v`, `payload`, `actor`, `caused_by`.
2. `stream_pos` is drawn from the same per-project counter as `agents.events`, and a test interleaving
   a domain entry and a telemetry event asserts one total order across both.
3. Appending an entry and updating its projections is one transaction — a test that fails the
   projection asserts the entry is not visible either.
4. A projection is never stale: no code path reads a projection while an appended entry is unapplied.
5. `agents.events` has **no** synchronous projection — a test asserts the telemetry append path touches
   one table.
6. `locus rebuild --into <scratch>` reproduces `board`, `workflows` and `mail` byte-identically to the
   live projections, from the log alone.
7. `locus rebuild` leaves carve-out columns untouched — proven by a rebuild that asserts embedding
   vectors are unchanged and were not recomputed.
8. `--to <stream_pos>` reproduces the board as of that point, and a test asserts a task shows its
   earlier column.
9. A fold meeting an unknown `(kind, v)` **halts with the offending stream_pos named**; it does not skip.
10. Every historical `(kind, v)` folds — a fixture per version, and adding a version without a fixture
    fails CI.
11. A non-foldable column without a `carve_out` annotation fails the schema test.
12. `locus backup` covers `log.entries`; a restore-then-rebuild drill produces the same projections.

## Open

- Whether `caused_by` is populated everywhere or only where a causal chain is actually queried. It is
  cheap to write and expensive to backfill, which argues for everywhere, but PLAN.md does not name a
  consumer for it yet.
- Log compaction. Nothing needs it at the scale of a single-user tool, and every scheme for it trades
  away `--to`. Deferred until a real volume number exists, not designed ahead of one.
