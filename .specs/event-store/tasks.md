# event-store — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Migration: `log` schema — `entries` with `stream_pos`, `kind`, `v`, `payload`, `actor`, `caused_by` | — | `cargo test -p locus-core store::schema_log` |
| 2 | `stream_pos` drawn from the same per-project counter as `agents.events` | 1 | `cargo test -p locus-core log::one_ordering_across_both_logs` |
| 3 | Interleave a domain entry and a telemetry event; assert one total order | 2 | `cargo test -p locus-core log::interleaved_total_order` |
| 4 | `kind` registry: a closed set per schema, each with its current `v` | 1 | `cargo test -p locus-core log::kind_registry` |
| 5 | Append + project in one transaction; a failing projection rolls back the entry | 4 | `cargo test -p locus-core log::append_is_atomic_with_fold` |
| 6 | Assert no read path touches a projection with an unapplied entry | 5 | `cargo test -p locus-core log::projections_never_stale` |
| 7 | Assert the telemetry append path writes one table and runs no projection | 5 | `cargo test -p locus-core log::telemetry_not_projected` |
| 8 | `board` projector: task, column, blocked, assignment, evidence | 5 | `cargo test -p locus-core project::board` |
| 9 | `workflows` projector: executions, iterations, guardrail trips, verify results | 5 | `cargo test -p locus-core project::workflows` |
| 10 | `mail` projector: threads, messages, delivery state | 5 | `cargo test -p locus-core project::mail` |
| 11 | `memory` and `wiki` projectors for the foldable columns only | 5,14 | `cargo test -p locus-core project::foldable_only` |
| 12 | `carve_out` column annotation and its registry | 1 | `cargo test -p locus-core carve_out::registry` |
| 13 | Schema test: a non-foldable column without `carve_out` fails | 12 | `cargo test -p locus-core carve_out::unannotated_fails` |
| 14 | Decay and confidence evaluated at read from `last_active` plus the curve | 12 | `cargo test -p locus-core carve_out::decay_read_time` |
| 15 | `locus rebuild --into <scratch>` replays the log into fresh projections | 8,9,10,11 | `cargo test -p locus-core rebuild::into_scratch` |
| 16 | Rebuild reproduces `board`, `workflows`, `mail` byte-identically to live | 15 | `cargo test -p locus-core rebuild::byte_identical` |
| 17 | Rebuild leaves carve-out columns untouched and recomputes no embedding | 15,12 | `cargo test -p locus-core rebuild::carve_outs_untouched` |
| 18 | `--to <stream_pos>` reproduces the board as of that point | 15 | `cargo test -p locus-core rebuild::time_travel` |
| 19 | `--schema <name>` rebuilds one projection without touching the others | 15 | `cargo test -p locus-core rebuild::single_schema` |
| 20 | Unknown `(kind, v)` halts the fold and names the offending `stream_pos` | 4,15 | `cargo test -p locus-core log::unknown_version_halts` |
| 21 | Assert the fold never skips an entry it cannot read | 20 | `cargo test -p locus-core log::never_skips` |
| 22 | Per-version fold fixtures; a new `v` without one fails CI | 4 | `cargo test -p locus-core log::every_version_folds` |
| 23 | `locus backup` covers `log.entries`; restore-then-rebuild drill matches | 15 | `cargo test -p locus-core backup::log_restore_rebuild_drill` |
