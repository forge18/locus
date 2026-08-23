# screens-automate

> **Historical M0.5 surface.** The fixture implementation is superseded by
> `.specs/task-orchestration/spec.md` and `.specs/external-work-items/spec.md` for live runtime work.

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · Views `board`, `tasks`

## Purpose

Automate shows project tasks. Kanban and List are two views of the same task set. A workflow execution,
root session, and its agents are details of a selected task, not peer-level Automate rows.

## Contract

### Kanban and List

Both tabs render the same six fixed-column task board: Ready · In Progress · Testing · Reviewing ·
Waiting For Approval · Done. Blocked remains an in-place status, never a column. The header provides
**New task** and **Import task** actions in both views; neither adds, removes, or reorders columns.

A card or row shows task summary, project/repo, workflow, root-session state, dependencies, verify state,
evidence count, and external-link state. Selecting it opens a task detail sheet, not an Agents pane.

### Task detail

The detail sheet shows the selected task's workflow, root orchestration session, child-agent run tree,
current activity, evidence, and run controls. Pause, cancel, handoff, guardrail, and needs-attention
actions remain task-scoped. The persistent running strip links to this sheet.

### Creation and import

New task opens the shared task draft and requires workflow confirmation before start. Import opens the
shared external-work-item preview and confirmation flow. Imported work becomes a local task and follows
the identical workflow/session/run path; no external write occurs before Done.

## Acceptance

1. Kanban and List contain tasks, not agents or sessions.
2. Both views expose the same New task and Import task actions and create the same drafts.
3. Both views resolve task selection to the same task detail sheet.
4. The task detail contains the task-owned workflow, root session, and agent run tree.
5. The six columns remain fixed and blocked stays orthogonal to column.

## Open

None.
