# repo-manager — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Bare local remote at `/var/lib/locus/repos/<project>.git` | — | `cargo test -p locus-core repo::bare_remote` |
| 2 | Add a linked repo, leaving the user's checkout untouched | 1 | `cargo test -p locus-core repo::add_linked` |
| 3 | Add a managed repo, cloned from GitHub | 1 | `cargo test -p locus-core repo::add_managed` |
| 4 | Shared object store for `--reference` clones | 1 | `cargo test -p locus-core repo::object_store` |
| 5 | Per-run clone into `/workspace` with `--reference` | 4 | `cargo test -p locus-core repo::run_clone` |
| 6 | Measure disk: N clones do not mean N histories | 5 | `cargo test -p locus-core repo::reference_saves_disk` |
| 7 | Branch naming `agent/<run-id>` | 5 | `cargo test -p locus-core repo::branch_naming` |
| 8 | Push the branch back to the bare remote | 7 | `cargo test -p locus-core repo::push_branch` |
| 9 | Assert no code path writes to `main` or `master` | 8 | `cargo test -p locus-core repo::never_writes_main` |
| 10 | A direct attempt to push `main` is refused | 9 | `cargo test -p locus-core repo::main_push_refused` |
| 11 | Merge-back when it merges cleanly | 8 | `cargo test -p locus-core repo::merge_back_clean` |
| 12 | Unresolvable conflict becomes an inbox item with both sides | 11 | `cargo test -p locus-core repo::conflict_to_inbox` |
| 13 | Involved repos cloned read-only to `/context/<repo>` | 5 | `cargo test -p locus-core repo::context_repos` |
| 14 | A push from a `/context/` repo fails | 13 | `cargo test -p locus-core repo::context_is_read_only` |
| 15 | Three concurrent agents on one repo, own clones, no interference | 5 | `cargo test -p locus-core repo::three_concurrent -- --ignored` |
| 16 | Decide and implement linked-repo sync timing | 2 | `cargo test -p locus-core repo::linked_sync` |
| 17 | Wire the Develop git panel to the real repo state | 8 | `pnpm -C apps/desktop test -- develop/git-from-core` |
