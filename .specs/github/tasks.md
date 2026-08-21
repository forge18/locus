# github — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `gh` wrapper in core with structured output | — | `cargo test -p locus-core gh::wrapper` |
| 2 | `gix` for local git operations | — | `cargo test -p locus-core gix::operations` |
| 3 | Branch operations against the bare remote | 2 | `cargo test -p locus-core gh::branch` |
| 4 | PR open, review and merge verbs | 1 | `cargo test -p locus-core gh::pr_verbs` |
| 5 | Assert Locus never merges to `main` itself | 4 | `cargo test -p locus-core gh::never_merges_main` |
| 6 | CI status fetch | 1 | `cargo test -p locus-core gh::ci_status` |
| 7 | Surface CI status on the board card | 6 | `pnpm -C apps/desktop test -- kanban/ci-status` |
| 8 | Surface CI status on the dashboard | 6 | `pnpm -C apps/desktop test -- status/ci-status` |
| 9 | Attach an existing issue, importing title, body and labels once | 1 | `cargo test -p locus-core gh::attach_issue` |
| 10 | Assert a later GitHub edit does not change the Locus task | 9 | `cargo test -p locus-core gh::no_background_sync` |
| 11 | Create a GitHub issue from a task, recording the link both ways | 1 | `cargo test -p locus-core gh::create_issue` |
| 12 | `Fixes #N` in the PR body when an issue is linked | 4,9 | `cargo test -p locus-core gh::pr_closes_issue` |
| 13 | Assert no polling loop or sync job exists | 10 | `bash scripts/check-no-github-polling.sh` |
| 14 | GitHub token routed through the credential broker | 1 | `cargo test -p locus-core gh::token_via_broker` |
