# memory — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `memory.core` bounded tier: hard cap, refuse-don't-evict | — | `cargo test -p locus-core memory::core_refuses_over_cap` |
| 2 | `memory.store` tier: facts, scope, provenance, embeddings, confidence, decay | — | `cargo test -p locus-core memory::store_schema` |
| 3 | Probation buffer, project-scoped | 2 | `cargo test -p locus-core memory::probation_is_project_scoped` |
| 4 | Assert there is no shared short-term tier | 3 | `cargo test -p locus-core memory::no_shared_short_term` |
| 5 | `locus-hook` capture path: append to a local buffer, background flush | — | `cargo test -p locus-cli hook::capture_is_async` |
| 6 | Assert the logging path never touches the socket synchronously | 5 | `cargo test -p locus-cli hook::never_blocks` |
| 7 | Injection path with a 100ms timeout emitting nothing on expiry | 5 | `cargo test -p locus-cli hook::injection_timeout` |
| 8 | Catalog builder: paths and one-line summaries, capped at 800 tokens | 2 | `cargo test -p locus-core memory::catalog_cap` |
| 9 | Over 40 entries triggers consolidation, never eviction | 8 | `cargo test -p locus-core memory::overflow_consolidates` |
| 10 | Catalog frozen at `SessionStart` for the run's life | 8 | `cargo test -p locus-core memory::catalog_is_frozen` |
| 11 | Assert catalog output never begins with `{` | 8 | `cargo test -p locus-core memory::no_leading_brace` |
| 12 | Materialize the catalog into `[layout].context` where a harness has no SessionStart hook | 8 | `cargo test -p locus-core memory::catalog_fallback_path` |
| 13 | Promotion check 1: re-verification against `codanna`, a test, or a verify result | 3 | `cargo test -p locus-core memory::reverification` |
| 14 | Promotion check 2: path-addressed dedup with subject-aware embedding first | 3 | `cargo test -p locus-core memory::dedup_by_path` |
| 15 | Promotion check 3: importance measured from injection, recall and verify outcome | 3 | `cargo test -p locus-core memory::importance_is_measured` |
| 16 | Cluster-density trigger: promote at the third candidate on a path | 13,14,15 | `cargo test -p locus-core memory::promotes_at_three` |
| 17 | Archive originals, never delete | 16 | `cargo test -p locus-core memory::archives_originals` |
| 18 | Buffer overflow promotes by score and logs every drop | 16 | `cargo test -p locus-core memory::overflow_logs_drops` |
| 19 | Symbol decay: signature changed invalidates | 2 | `cargo test -p locus-core memory::signature_change_invalidates` |
| 20 | Symbol decay: body changed flags for re-verification | 19 | `cargo test -p locus-core memory::body_change_flags` |
| 21 | Symbol decay: AST unchanged changes nothing | 19 | `cargo test -p locus-core memory::ast_stable_is_noop` |
| 22 | Ebbinghaus curve with the four half-lives and `active_days` | 2 | `cargo test -p locus-core memory::decay_curve` |
| 23 | Chain-aware pruning: a strong neighbour keeps a decayed memory alive | 22 | `cargo test -p locus-core memory::chain_aware_pruning` |
| 24 | Cold-start guard: never prune what never had a chance to match | 22 | `cargo test -p locus-core memory::cold_start_guard` |
| 25 | Recall: role and visibility filter, then BM25 + vector hybrid, then graph expansion | 2 | `cargo test -p locus-core memory::hybrid_recall` |
| 26 | Rank by similarity x strength | 25 | `cargo test -p locus-core memory::rank_by_similarity_times_strength` |
| 27 | Task-class retrieval depth: k=1 flat for code and plan, high with graph for research | 25 | `cargo test -p locus-core memory::depth_by_task_class` |
| 28 | Turn-level injection off for `code` and `plan` | 27 | `cargo test -p locus-core memory::no_turn_injection_for_code` |
| 29 | The keeper as an ordinary agent definition at `high` tier | 16 | `cargo test -p locus-core memory::keeper_is_an_agent` |
| 30 | Genuine project idle as the keeper trigger | 29 | `cargo test -p locus-core memory::keeper_triggers_on_idle` |
| 31 | Keeper pass: read since watermark, cluster, check, merge, recompute, prune, archive | 29 | `cargo test -p locus-core memory::keeper_pass` |
| 32 | Assert the primary agent has no memory-edit tools unless `write: direct` | 29 | `cargo test -p locus-core memory::primary_cannot_edit` |
| 33 | `locus memory note add\|replace\|remove` | 1 | `cargo test -p locus-cli memory::note_verbs` |
| 34 | `locus memory recall`, `write`, `forget` | 25 | `cargo test -p locus-cli memory::store_verbs` |
| 35 | Empty store degrades to the no-memory baseline | 8,25 | `cargo test -p locus-core memory::empty_store_degrades_cleanly` |
| 36 | Cross-harness: one harness writes, a different one recalls | 34 | `cargo test -p locus-core memory::cross_harness_recall -- --ignored` |
