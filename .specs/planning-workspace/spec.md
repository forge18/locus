# planning-workspace

**Status:** Accepted · **Milestone:** Workstream 4 · **Depends on:** `plan-revision`, `design-revision`, `shell-revision`

## Purpose

Planning is a durable workspace above the existing bounded planning flow. It can plan an amendment,
feature, or whole project; persist through application and agent replacement; and end with a reviewed,
approved package of specs and tasks. This contract governs the workspace wrapper, page-owned navigation,
and the retirement of the former Interact surface.

## Governed by

- [`PLAN.md`](../../PLAN.md) — architecture and adopted navigation
- [`PLANNING_WORKSPACE_PLAN.md`](../../PLANNING_WORKSPACE_PLAN.md) — accepted product contract
- `.specs/plan-revision/spec.md` — seven-stage child-spec profile
- `.specs/agent-definitions/spec.md` — immutable agent definition and run pinning

## Contract

### Workspace shape

A `PlanningWorkspace` belongs to exactly one project and has a scope of `amendment`, `feature`, or
`project`. It contains a brief, use cases, success criteria, decisions, risks, open questions,
research, child `WorkspaceSpec` records, cross-spec dependencies, planning-session links, revisions,
and materialization records. `core.plans` remains the bounded leaf projection; it is not made recursive.
Each child spec names exactly one writable target repository.

### Lifecycle and checkpoints

```text
Draft → In progress → Ready for approval → Approved
  └────────────────────────────────────────→ Deleted
```

Lifecycle is separate from agent presence. Leaving a page never changes lifecycle. An in-progress
workspace freezes its current revision as a resumable checkpoint and creates no board task. Returning
users and replacement agents resume from the checkpoint, pending decisions, active child spec, and
structured handoff rather than from transcript text.

Deletion is allowed only for a draft or in-progress workspace and is a hard delete. Approved revisions
and their provenance are never deleted. Review always asks the user to choose a project before loading
project QA.

### Revisions and approval

Autosave and background work use optimistic revision checks. Shared decisions mark affected child
specs stale instead of silently rewriting them. Approval is one transactional, idempotent operation:

1. lock the workspace and verify the requested revision is current;
2. require reviewed specs, complete requirement-to-task coverage, owned repositories, and an acyclic
   cross-spec dependency graph;
3. freeze the exact revision;
4. create **all** approved board tasks, dependency edges, and provenance records;
5. return the existing materialization on a retry rather than duplicating board work.

No task reaches the board while a workspace is incomplete or merely checkpoint-frozen.

### Navigation and scope

The production route manifest is the route authority; fixtures derive from it and do not define routes.
The rail has Projects, Workers, Telemetry, Automation (Plan, Manage, Review), and Workshop (Extensions,
Plugins, Knowledge, Settings). There is no global project selector. Projects, Workers, and Telemetry
support all-project views; Plan, Manage, Review, Knowledge, Inbox, and Dispatch own explicit local
filters. Object locators remain project-owned, and compatible non-retired deep links continue to resolve.

The former Interact route is removed. Its disposable board-less sessions, changed-file research,
commit, promote, and discard actions are explicitly retired, not silently moved to another page.
Workers owns one durable home conversation per named agent, its branch, routines, and warm-window state.
No second disposable session type is introduced.

### Capability policy

Workshop → Extensions → Agents independently resolves CLI tools, commands, and skills. The effective
set is the non-escalating intersection of project policy, agent restriction, and workflow/task
restriction. `AllowOnly([])` means none; an absent restriction means inherit. Runs record policy
revisions and resolved sets. A project change affects future runs only.

## Supersedes

| Existing contract | Replacement scope |
| --- | --- |
| `.specs/design-revision/spec.md` | rail vocabulary and screen inventory |
| `.specs/shell-revision/spec.md` | global project selector and category map |
| `.specs/interact-sessions/spec.md` | entire surface; Interact is retired |
| `.specs/bots/spec.md` | route label and session taxonomy; durable behavior is renamed Workers |
| `.specs/plan-revision/spec.md` | workspace wrapper and approval/materialization policy; its seven-stage child profile remains |
| `.specs/desktop-data-integration/spec.md` | legacy global-scope and route assumptions; live/demo boundary remains |

The historical contracts retain their implementation details only where this table says they remain.

## Acceptance

1. A workspace belongs to one project and validates `amendment`, `feature`, or `project` scope.
2. Workspace lifecycle and agent presence are stored separately; leaving a page does not end work.
3. An incomplete workspace freezes a resumable checkpoint without creating board tasks.
4. Reopening after application or agent restart restores structured state, pending decisions, and the
   active child spec without treating a transcript as canonical state.
5. Approval rejects incomplete coverage, unowned repositories, and cyclic dependencies.
6. Approval freezes one revision and creates all approved board tasks exactly once on retries.
7. The route manifest, not fixtures, is production navigation authority.
8. No global project selector exists; page-owned filters do not leak across surfaces.
9. The rail contains Projects, Workers, Telemetry, Automation, and Workshop with the subgroups named
   above; Analytics is under Telemetry and Autorun/Schedules/Runs remain under Dispatch.
10. Interact is absent from rail items, route ids, and deep-link resolution; its retired capabilities
    are documented rather than silently reassigned.
11. Workers provides one durable home conversation per named agent and does not expose a disposable
    session type.
12. Effective agent capabilities never exceed project policy, and each run stores the resolved policy
    snapshot.

## Verification

```bash
rg -q '^\*\*Status:\*\* Accepted' .specs/planning-workspace/spec.md
rg -q 'creates \*\*all\*\* approved board tasks' .specs/planning-workspace/spec.md
rg -q 'There is no global project selector' .specs/planning-workspace/spec.md
rg -q 'Interact is absent' .specs/planning-workspace/spec.md
```
