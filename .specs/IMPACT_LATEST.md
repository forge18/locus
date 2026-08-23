# Forge-provider support impact

## Target

The planned M7 GitHub integration: forge-hosted change requests, issues, CI status, review comments,
authentication, and repository metadata. Local Git operations remain `gix` and do not become
provider-specific.

## Dependents (8)

- `PLAN.md:2568` — M7 currently specifies GitHub-only `gh` + `gix` operations.
- `.specs/forge-providers/spec.md` and `tasks.md` — the provider-neutral successor to the 14 unimplemented GitHub-specific tasks.
- `.specs/agent-prs/spec.md` and `tasks.md` — review-comment delivery currently names GitHub.
- `.specs/ci-babysitter/spec.md` — CI log/status API needs a provider contract.
- `.specs/repo-manager/spec.md` — managed-repository import says GitHub.
- `.specs/board/spec.md` — task contract now names `external_issue`.
- `migrations/0003_board_schema.up.sql` — `board.github_issues` is the only implemented persisted
  GitHub-specific model.
- `crates/locus-core/src/store.rs` — schema test asserts `github_issues`; desktop fixture/types
  contain presentation-only GitHub wording.

## Affected Stories

- M3 `repo-manager` — discover and record forge host/project identity while importing managed repos.
- M5 `board` — replace the GitHub-only link with a provider-neutral external issue link.
- M7 `github` — replace with a forge-provider feature covering GitHub, GitLab, Bitbucket Cloud, and
  Codeberg. `agent-prs` and `ci-babysitter` depend on it.

## Test Coverage

- `store::schema_board` covers the present `github_issues` table only.
- No provider, PR/MR, issue, CI, review-comment, webhook, or authentication implementation exists yet:
  M7 has zero completed tasks.
- No compatibility migration test exists for moving a GitHub link to a provider-neutral record.

## Risk: High

The implementation footprint is small today, but the integration is a shared contract across repository
import, board links, credentials, CI, and session comments. Adding GitHub first and abstracting later
would force a second public-API and database migration.

## Recommended action

Define a provider-neutral `ForgeProvider` port before M7 implementation. Keep local Git in `gix`; give
provider adapters normalized issue, change-request, check, review-comment, and webhook operations.
Resolve adapters from persisted forge host and project identity, keep credential references host-only,
and require explicit capability declarations instead of assuming every forge has every feature. Implement
separate Bitbucket Cloud and Bitbucket Data Center adapters: their APIs and authentication differ.
Migrate `board.github_issues` to a provider-neutral external-issue table before implementing any provider.
