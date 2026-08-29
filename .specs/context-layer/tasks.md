# context-layer — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `eviction_class` column on memory records: `sticky` \| `standard`, default `standard`, folded from the log per the carve-out rules | — | `cargo test -p locus-core memory::eviction_class_schema` |
| 2 | Assignment at capture/promotion: error-derived records and R2 declarations are `sticky`; all others `standard` | 1 | `cargo test -p locus-core memory::eviction_class_assignment` |
| 3 | Catalog overflow drops `standard` by strength, never `sticky`, and logs every drop | 1 | `cargo test -p locus-core memory::catalog_overflow_respects_eviction_class` |
| 4 | Derivation table: (knowledge-kind, lifetime, task_class) → placement/injection/eviction, pure and byte-deterministic across runs | — | `cargo test -p locus-core materializer::context_derivation_table` |
| 5 | Derivation table maps constitution and rules to the always/never-dropped slot outside the memory store; assembled order unchanged for existing fixtures | 4 | `cargo test -p locus-core materializer::derivation_keeps_constitution_head` |
| 6 | `locus memory promote --json`: skips density, runs re-verification and dedup, lands in probation metadata | 1 | `cargo test -p locus-cli memory::promote_verb` |
| 7 | Promoted-without-density records decay exactly like any other memory | 6 | `cargo test -p locus-core memory::promoted_decay_normally` |
| 8 | Research-class selection suppresses candidates above the similarity threshold to an already-selected candidate (MMR-lite); threshold is a setting | 25* | `cargo test -p locus-core memory::research_diversity_dedup` |
| 9 | `code`/`plan` selection is byte-identical to the pre-R4 pipeline | 8 | `cargo test -p locus-core memory::k1_selection_unchanged` |
| 10 | Tail section: append-only "new since snapshot" entries (paths + one-liners) at the mutable tail; budget derived from the effective-window derivation, appends capacity-aware (check remaining capacity, compress/drop tail first per ContextBudget), overflow logged | 1 | `cargo test -p locus-core memory::catalog_tail_append` |
| 11 | Head byte-stability: the frozen snapshot's bytes are identical with and without tail entries; tail entries never begin a line with `{` | 10 | `cargo test -p locus-core memory::snapshot_head_stable` |
| 12 | Recitation block: ≤3 lines (objective, step, unresolved-error count) emitted by the run supervisor on task-state changes, tail-placed, absent without a plan, never touching the frozen head, no model call | — | `cargo test -p locus-core run_supervisor::recitation_block` |
| 13 | Calibration-loop cache-rate acceptance: paired-run comparison arms with a cache-rate non-regression criterion from `usage.cache_read` / `usage.input` | — | `cargo test -p locus-core calibration::cache_rate_criterion` |
| 14 | `context_attribution` view: injection/recall events ↔ verify outcomes ↔ `tool_result` rows joined with the per-run materialization snapshot, plus verification-cost columns; view only, no backfill | — | `cargo test -p locus-core telemetry::attribution_view` |

\* Task 8 depends on the existing hybrid recall pipeline (memory tasks 25–27), complete.

## Notes

- Task 11 coexists with memory task 10 ("catalog frozen at `SessionStart`"): the snapshot
  stays frozen; the tail section is a separate append-only structure. Both tests must pass
  simultaneously — that is the point of R5.
- Task 12's recitation block rides the existing injection path and inherits its 100ms
  timeout and exit-0-everywhere discipline; it adds no hook events.
- Task 13 amends calibration-loop (TODO item 25, complete) in place; its acceptance
  criterion is computed from columns that already exist on every event.
- Task 14 is view-only by contract; if the view cannot later disambiguate mid-run rule
  loads, a correlation id becomes a revision, not a change to this spec.
- Docker-gated integration coverage follows each amended feature's existing notes; these
  tasks add no new Docker dependencies.
- Batch evidence: [verification.md](verification.md) records all 14 task checks and the final project gates.
