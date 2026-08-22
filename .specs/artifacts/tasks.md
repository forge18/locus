# artifacts — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Artifact row: kind, text body or blob path, media type, `sha256`, derived cache | — | `cargo test -p locus-core artifact::row` |
| 2 | Kind enum split into review and reference groups | 1 | `cargo test -p locus-core artifact::kind_groups` |
| 3 | Assert reference kinds are excluded from inbox queries | 2 | `cargo test -p locus-core artifact::reference_never_in_inbox` |
| 4 | Text kinds stored in Postgres | 1 | `cargo test -p locus-core artifact::text_is_a_row` |
| 5 | Blob tree under `/var/lib/locus/artifacts/<project>/<run>/` | 1 | `cargo test -p locus-core artifact::blob_tree` |
| 6 | `sha256` computed and stored on write | 5 | `cargo test -p locus-core artifact::sha256` |
| 7 | `locus artifact put <kind> <path>` | 4,5 | `cargo test -p locus-cli artifact::put` |
| 8 | `locus artifact get <id>` round-trips unchanged | 7 | `cargo test -p locus-cli artifact::get_roundtrip` |
| 9 | Comment threads attached to an artifact | 1 | `cargo test -p locus-core artifact::comment_threads` |
| 10 | A comment routes into the live session that produced the artifact | 9 | `cargo test -p locus-core artifact::comment_steers_live` |
| 11 | A comment after the last run exited is delivered at next run start | 10 | `cargo test -p locus-core artifact::comment_deferred_delivery` |
| 12 | `locus artifact comments` listing feedback on this run's artifacts | 9 | `cargo test -p locus-cli artifact::comments` |
| 13 | Compaction hook: over-threshold tool results become `payload` artifacts | 7 | `cargo test -p locus-core artifact::compacts_overflow` |
| 14 | Leave a one-line summary and an id in place of the body | 13 | `cargo test -p locus-core artifact::summary_with_handle` |
| 15 | Assert the summary is materially smaller than the body | 14 | `cargo test -p locus-core artifact::summary_ratio` |
| 16 | Threshold as a setting with a default, not a constant | 13 | `cargo test -p locus-core artifact::threshold_is_a_setting` |
| 17 | Walkthrough generated from a finished session, inlining its artifacts | 4 | `cargo test -p locus-core artifact::walkthrough_generates` |
| 18 | Retention: media pruned at 30 days unless linked to a PR or a Done task | 5 | `cargo test -p locus-core artifact::media_retention` |
| 19 | Assert text artifacts are never pruned | 18 | `cargo test -p locus-core artifact::text_never_pruned` |
| 20 | Backup covers the blob tree; restored paths resolve | 5 | `cargo test -p locus-core artifact::backup_covers_blobs` |
| 21 | Wire the Review Artifacts screen to real artifacts over IPC | 8,12 | `pnpm -C apps/desktop test -- artifacts/from-core` |
