# store

**Milestone** M1 · **Depends on** none · **Blocks** every M1 feature and everything after

## Purpose

Postgres as the single source of truth, plus the backup that makes that safe. PLAN.md is blunt about
the consequence of its own design: with board, wiki, memory and mail living **only** in Postgres,
losing the volume loses work no reindex can rebuild. Backup is therefore a requirement here, not an
operational afterthought — and it is the one item PLAN.md's deferral table calls non-deferrable.

## Governed by

- PLAN.md §Data model — the eight schemas and their boundaries
- PLAN.md §Containers — `locus-postgres`, per machine, `pgvector` + `tsvector` + window functions
- PLAN.md §M1 — backup/restore with a drill
- PLAN.md §Risks — "Risk — Postgres is now irreplaceable"

## Contract

**Eight schemas**, `sqlx` migrations in `migrations/`:

| Schema | Holds |
| --- | --- |
| `core` | projects, repos, local remotes, settings |
| `agents` | agent_defs (versioned), sessions, runs, run edges, events, artifacts, comment threads |
| `board` | tasks, dependency edges, transitions, assignments, task-run links, evidence, GitHub issues |
| `wiki` | pages (typed), revisions, links, contradictions, ingest log, embeddings |
| `memory` | core (bounded) and store (facts, scope, provenance, embeddings, confidence, decay) |
| `workflows` | workflow_defs (versioned), schedules, executions, iterations, guardrail trips, verify results |
| `mail` | threads, messages, delivery state |
| `market` | manifests, installs, per-image tool sets |

**Which is derived and which is not.** Harness output, git, and the marketplace index are sources of
truth and are never written by Locus. **Board, wiki, memory and mail exist only here** — they are
backed up rather than rebuilt. That asymmetry is the whole reason backup is in M1.

**Event bus**: in-process broadcast plus Postgres `LISTEN/NOTIFY` across processes. **NOTIFY carries an
id only** — the payload cap is 8000 bytes, so anything larger is fetched by the listener.

**Backup**:
```
locus backup                 dumps the eight schemas AND the artifact blob tree together
locus restore --drill        restores into a scratch database, asserts row counts against the source
```
Nightly and before every migration. Seven dailies, four weeklies.

**`--drill` is the point.** A backup nobody has restored is a belief, not a backup — and it is why
restore is a first-class verb rather than documentation.

**Backup covers both trees or neither.** Text artifacts are rows; media is files under
`/var/lib/locus/artifacts/`. A backup that takes one and not the other restores a database full of
paths to nothing.

## Acceptance

1. All eight schemas exist as `sqlx` migrations and apply to an empty database in one run.
2. Migrations are reversible or explicitly marked one-way with a reason in the file.
3. `pgvector` and `tsvector` are available and a round-trip embedding query returns a result.
4. `locus backup` produces an artifact containing both the SQL dump and the blob tree.
5. `locus restore --drill` restores into a scratch database and **fails loudly** on a row-count
   mismatch — proven by a test that corrupts a dump on purpose.
6. A migration run triggers a backup first, and the backup's completion gates the migration.
7. `NOTIFY` payloads are ids only; a test asserts none exceeds the 8000-byte cap.

## Open

- PLAN.md defers detailed table definitions for six of the eight schemas, keeping full tables only for
  `memory` and `board`. The rest get written properly with their migration — this spec does not
  pre-empt that, and each consuming feature's spec carries its own tables.
