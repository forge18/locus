# forge-providers

**Milestone** M7 · **Depends on** `repo-manager`, `board`

## Purpose

Connect Locus to GitHub, GitLab, Codeberg, Bitbucket Cloud, and Bitbucket Data Center for hosted
change requests, CI, and explicitly linked issues. Locus projects remain unrelated to provider project
features.

## Governed by

- PLAN.md §M7 — version control, CI/CD, and agent-authored PRs
- PLAN.md §The board — externally linked issue on a task
- `.specs/repo-manager/spec.md` — local bare remotes and the no-`main` invariant

## Contract

Local repository work stays in `gix`. A provider-neutral `ForgeProvider` port owns only remote-forge
operations: repository identity, issue import/create, change-request open/update, CI status and logs,
review comments, signed webhook verification, and an `ExternalWorkItemProvider` bridge for import.

The adapter registry selects an implementation from a persisted provider kind and host/base URL:

| Provider kind | Implementation |
| --- | --- |
| GitHub | GitHub API adapter; configured for GitHub.com or Enterprise host |
| GitLab | GitLab API adapter; configured for GitLab.com or self-managed host |
| Codeberg | Forgejo API adapter fixed to Codeberg's host |
| Bitbucket Cloud | Bitbucket Cloud API adapter |
| Bitbucket Data Center | Bitbucket Server/Data Center API adapter |

Every adapter declares capabilities. A UI or workflow requests a capability, not a provider name, and
receives a clear refusal when the configured provider cannot perform it. Provider CLI tools (`gh`,
`glab`, `bb`) are not the Locus API: their installation, authentication, and output are not a stable
cross-provider contract.

**Issues are explicitly linked once.** Attaching imports title, body, labels, URL, native identifier,
provider kind, host, and repository identity at that moment. Creating an issue records the same link.
Nothing polls or synchronizes the issue afterward. Provider-specific close-reference syntax is generated
by the adapter; it is never hard-coded as `Fixes #N`.

**Credentials stay host-only.** A provider credential is keyed by provider kind and host/base URL, is
retrieved through the credential broker, and never enters a container or persisted configuration.

**Incoming events are webhooks.** Signed review-comment and CI events are verified by the selected
adapter, made durable, and routed into the existing session-comment and CI-babysitter paths. No polling
loop is introduced.

**Reaching `main` is a human action through a change request.** Locus can open, update, and comment on
a change request, but never merges to `main` or `master`.

## Acceptance

1. `gix` performs local Git work; no provider adapter owns clone, branch, or local merge operations.
2. All five provider kinds resolve from persisted provider kind and host/base URL.
3. Each adapter passes the same contract suite for supported capabilities and refuses unsupported ones.
4. Attaching an external issue imports its snapshot once; a later provider-side edit never changes the
   Locus task.
5. Creating an external issue records a provider-neutral link in both directions.
6. Change-request text uses the selected adapter's close-reference syntax when an external issue is linked.
7. CI status appears on the board card and dashboard through normalized check data.
8. Signed review-comment webhooks route through the same code path as artifact comments.
9. Signed CI webhooks reach the CI babysitter without a polling job.
10. Provider tokens are obtained only through the credential broker and are scoped by provider host.
11. No Locus path merges `main` or `master`.
12. Existing GitHub issue links migrate without losing their repository, number, URL, or snapshot.
13. Each forge adapter exposes its issue tracker through the external-work-item provider contract.

## Open

None.
