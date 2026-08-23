# external-work-items

**Milestone** M7 · **Depends on** `task-orchestration`, `forge-providers`, `board`

## Purpose

Import work from GitHub, GitLab, Codeberg, Bitbucket Cloud, Bitbucket Data Center, Jira, and future
trackers into the same local task workflow as manually created work. Locus owns execution after import;
the source system is not synchronized while work is in progress.

## Contract

An `ExternalWorkItemProvider` port normalizes provider kind, host/base URL, workspace or project,
native ID, URL, title, body, labels, status, and completion capabilities. Forge adapters expose it for
their issue trackers; Jira has its own adapter. Later trackers add an adapter without changing task,
workflow, or Automate code.

Import is explicit. The user chooses a configured provider and item, previews the source snapshot and
workflow, and confirms. Locus creates one local task with the imported snapshot and external identity,
then follows the identical task-owned workflow/session/run model as manual work. Duplicate import of
the same provider identity is refused or opens the existing local task.

**No outbound update occurs before local Done.** Locus does not poll, synchronize edits, transition
status, comment, or otherwise write to the source item while the task is Ready, In Progress, Testing,
Reviewing, Waiting For Approval, blocked, paused, cancelled, or failed.

On the first transition to local Done, Locus emits one durable completion delivery. It posts a concise
completion comment with task locator and evidence, then resolves/completes the external item if that
provider declares the capability. The delivery is idempotent and retryable after a transport failure;
retries deliver only this completion event, never a source synchronization. A provider without a
completion transition receives the comment and records that resolution is unsupported.

## Acceptance

1. Manual and imported tasks use the same task, workflow, root-session, run-tree, and Automate-detail contracts.
2. The import surface lists configured work-item providers and previews an item before creating a task.
3. GitHub, GitLab, Codeberg, Bitbucket Cloud, Bitbucket Data Center, and Jira adapters import a normalized snapshot.
4. A later edit to the source item never changes the local task.
5. Duplicate provider identity import opens the existing task and creates no second task.
6. No source write occurs until the task reaches Done; tests cover every non-Done local state.
7. Done emits exactly one idempotent completion comment containing the task locator and evidence.
8. Done resolves the external item when supported; unsupported resolution is visible without failing local Done.
9. A failed completion delivery retries the same durable completion event without fetching or synchronizing the source item.
10. A new provider adapter passes the work-item conformance suite without changes to Automate or task orchestration.

## Open

None.
