# forge-providers — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Define provider-neutral forge types and capability contract | — | `cargo test -p locus-core forge::contract_types` |
| 2 | Keep local clone, branch, and merge operations in `gix` | 1 | `cargo test -p locus-core forge::local_git_stays_in_gix` |
| 3 | Persist forge kind, host/base URL, and repository identity | 1 | `cargo test -p locus-core forge::repository_identity_roundtrip` |
| 4 | Migrate `board.github_issues` to provider-neutral external issue links | 3 | `cargo test -p locus-core forge::migrates_github_issue_links` |
| 5 | Select an adapter by persisted forge kind and host | 1,3 | `cargo test -p locus-core forge::adapter_selection` |
| 6 | Declare and enforce per-adapter capabilities | 1,5 | `cargo test -p locus-core forge::capabilities_refuse_unsupported_operations` |
| 7 | Normalize issue snapshots and explicitly link them to board tasks | 3,5 | `cargo test -p locus-core forge::attach_issue_once` |
| 8 | Assert provider-side issue edits never synchronize into Locus | 7 | `cargo test -p locus-core forge::no_issue_background_sync` |
| 9 | Create a linked external issue from a board task | 7 | `cargo test -p locus-core forge::create_issue` |
| 10 | Generate provider-specific issue close references | 7 | `cargo test -p locus-core forge::close_reference` |
| 11 | Normalize change-request state and review metadata | 1,5 | `cargo test -p locus-core forge::change_request_normalization` |
| 12 | Normalize CI checks and log retrieval | 1,5 | `cargo test -p locus-core forge::ci_normalization` |
| 13 | Surface normalized CI status on the board card | 12 | `pnpm -C apps/desktop test -- kanban/ci-status` |
| 14 | Surface normalized CI status on the dashboard | 12 | `pnpm -C apps/desktop test -- status/ci-status` |
| 15 | Route provider credentials through the host broker by forge host | 3,5 | `cargo test -p locus-core forge::token_via_broker` |
| 16 | Verify provider webhook signatures before accepting review events | 5,15 | `cargo test -p locus-core forge::review_webhook_signature` |
| 17 | Route review-comment webhooks to the artifact comment path | 16 | `cargo test -p locus-core forge::review_comment_routes_to_session` |
| 18 | Verify provider webhook signatures before accepting CI events | 5,15 | `cargo test -p locus-core forge::ci_webhook_signature` |
| 19 | Route CI webhooks to the CI babysitter without polling | 18 | `cargo test -p locus-core forge::ci_event_starts_babysitter` |
| 20 | Assert no provider polling loop or sync job exists | 8,19 | `bash scripts/check-no-forge-polling.sh` |
| 21 | Implement the GitHub adapter for GitHub.com and Enterprise hosts | 5,6 | `cargo test -p locus-core forge::github_contract` |
| 22 | Implement the GitLab adapter for GitLab.com and self-managed hosts | 5,6 | `cargo test -p locus-core forge::gitlab_contract` |
| 23 | Implement the Codeberg Forgejo adapter | 5,6 | `cargo test -p locus-core forge::codeberg_contract` |
| 24 | Implement the Bitbucket Cloud adapter | 5,6 | `cargo test -p locus-core forge::bitbucket_cloud_contract` |
| 25 | Implement the Bitbucket Data Center adapter | 5,6 | `cargo test -p locus-core forge::bitbucket_data_center_contract` |
| 26 | Run the shared provider conformance suite against every adapter | 21,22,23,24,25 | `cargo test -p locus-core forge::conformance` |
| 27 | Open and update a change request without merging protected branches | 11,26 | `cargo test -p locus-core forge::never_merges_main` |
| 28 | Store and expose provider change-request links for agent PRs | 11,26 | `cargo test -p locus-core forge::change_request_links` |
| 29 | Preserve external-link migration compatibility on store reconnect | 4 | `cargo test -p locus-core forge::links_survive_reconnect` |
| 30 | Exercise each adapter against its recorded HTTP contract fixtures | 21,22,23,24,25 | `cargo test -p locus-core forge::recorded_contracts` |
