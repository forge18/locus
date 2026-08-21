# agent-prs — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Generate a PR description from goal, closed tasks and evidence | — | `cargo test -p locus-core pr::description_from_session` |
| 2 | Assert the description is not a diff summary | 1 | `cargo test -p locus-core pr::not_a_diff_summary` |
| 3 | Attach `locus browse` screenshots to the PR | 1 | `cargo test -p locus-core pr::attaches_screenshots` |
| 4 | Self-review pass over the agent's own diff | — | `cargo test -p locus-core pr::self_review` |
| 5 | Apply self-review fixes before the PR is offered | 4 | `cargo test -p locus-core pr::second_draft` |
| 6 | Self-review findings visible on the PR | 4 | `cargo test -p locus-core pr::findings_visible` |
| 7 | Slicing threshold, decided and implemented | 1 | `cargo test -p locus-core pr::slice_threshold` |
| 8 | Slice a large change into independently reviewable PRs | 7 | `cargo test -p locus-core pr::slices` |
| 9 | Route a GitHub review comment into the authoring session | — | `cargo test -p locus-core pr::comment_routes_to_session` |
| 10 | Assert it is the same code path as artifact comments | 9 | `cargo test -p locus-core pr::one_comment_implementation` |
| 11 | Agent pushes a follow-up commit and replies | 9 | `cargo test -p locus-core pr::follow_up_commit` |
| 12 | A comment after the last run exited is delivered at next run start | 9 | `cargo test -p locus-core pr::deferred_comment` |
| 13 | Merge conflict returned as a proposed resolution with both sides | — | `cargo test -p locus-core pr::proposes_resolution` |
| 14 | Accept or reject a proposed resolution | 13 | `cargo test -p locus-core pr::accept_reject_resolution` |
