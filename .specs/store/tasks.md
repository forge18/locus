# store — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `locus-postgres` container lifecycle: up, down, health, with `pgvector` | — | `cargo test -p locus-core store::container_lifecycle` |
| 2 | `sqlx` wiring, connection pool, and the migration runner | 1 | `cargo test -p locus-core store::migrate_runs` |
| 3 | Migration: `core` — projects, repos, local remotes, settings | 2 | `cargo test -p locus-core store::schema_core` |
| 4 | Migration: `agents` — agent_defs, sessions, runs, edges, events, artifacts, comments | 2 | `cargo test -p locus-core store::schema_agents` |
| 5 | Migration: `board` — full tables per PLAN.md §The board | 2 | `cargo test -p locus-core store::schema_board` |
| 6 | Migration: `wiki` — pages, revisions, links, contradictions, ingest log, embeddings | 2 | `cargo test -p locus-core store::schema_wiki` |
| 7 | Migration: `memory` — full tables per PLAN.md §Memory, both tiers | 2 | `cargo test -p locus-core store::schema_memory` |
| 8 | Migration: `workflows` — defs, schedules, executions, iterations, trips, verify results | 2 | `cargo test -p locus-core store::schema_workflows` |
| 9 | Migration: `mail` — threads, messages, delivery state | 2 | `cargo test -p locus-core store::schema_mail` |
| 10 | Migration: `market` — manifests, installs, per-image tool sets | 2 | `cargo test -p locus-core store::schema_market` |
| 11 | Apply all eight to an empty database in one run | 3,4,5,6,7,8,9,10 | `cargo test -p locus-core store::migrate_from_empty` |
| 12 | Every migration is reversible or carries a one-way reason | 11 | `cargo test -p locus-core store::migrations_reversible_or_explained` |
| 13 | `pgvector` round trip: insert an embedding, query nearest | 11 | `cargo test -p locus-core store::pgvector_roundtrip` |
| 14 | `tsvector` full-text round trip | 11 | `cargo test -p locus-core store::fts_roundtrip` |
| 15 | Event bus: in-process broadcast | 2 | `cargo test -p locus-core bus::in_process` |
| 16 | Event bus: `LISTEN/NOTIFY` across processes, ids only | 15 | `cargo test -p locus-core bus::notify_across_processes` |
| 17 | Assert no NOTIFY payload exceeds 8000 bytes | 16 | `cargo test -p locus-core bus::notify_payload_cap` |
| 18 | `locus backup` — SQL dump plus the artifact blob tree in one artifact | 11 | `cargo test -p locus-core backup::covers_both_trees` |
| 19 | Retention: seven dailies, four weeklies | 18 | `cargo test -p locus-core backup::retention` |
| 20 | `locus restore` into a scratch database | 18 | `cargo test -p locus-core restore::into_scratch` |
| 21 | `--drill` asserts row counts against the source | 20 | `cargo test -p locus-core restore::drill_asserts_counts` |
| 22 | Drill fails loudly on a deliberately corrupted dump | 21 | `cargo test -p locus-core restore::drill_detects_corruption` |
| 23 | A migration triggers a backup first and is gated on its completion | 18,11 | `cargo test -p locus-core backup::gates_migration` |
