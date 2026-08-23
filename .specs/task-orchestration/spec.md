# task-orchestration

**Milestone** M5 · **Depends on** `board`, `workflow-engine`, `run-supervisor`, `screens-automate`

## Purpose

Automate is task-centric. A task owns its workflow execution and root session; agents are workers inside
that task, including child agents invoked by the orchestration agent. Kanban and List are alternate
views of the same task set, never task and agent lists that happen to share a title.

## Contract

```
task
└── workflow execution / root session
    └── agent runs, including subagents
```

A task has one selected workflow before it can start. Manual creation from Kanban or List creates the
same task draft; the project-default workflow may be preselected, but the creator confirms it. Starting
the task creates the root orchestration session. Retries and loop resets create new runs in that session;
subagents remain children of the task's root execution.

Kanban and List show the same project-scoped tasks, their column, workflow, root-session state, worker
summary, dependencies, evidence, and external-link state. Selecting a task opens an in-place task detail
sheet with its workflow, root session, run tree, current agent activity, controls, and evidence. There
is no peer-level Automate Agents list: agents are viewed in their task context. The persistent running
strip remains a cross-project summary and links to the owning task.

Pause, cancel, handoff, guardrail, and needs-attention controls target a run through its owning task;
they never detach a run from its task or expose an unowned agent work queue.

## Acceptance

1. Kanban and List render the identical task set and resolve to the same `locus://<project>/task/<id>` locator.
2. Neither view renders agents or sessions as its primary rows or cards.
3. Manual creation is available from both views and produces the same task draft and audit entries.
4. A workflow is selected and confirmed before a manually created task can start.
5. Starting a task creates its root workflow execution and session; a root session cannot exist without its task.
6. The task detail sheet shows the workflow, root session, child-agent run tree, controls, evidence, and external link.
7. Loop resets and subagent calls remain linked to the task through the root session.
8. A control initiated from a task applies only to that task's owned run tree.
9. The running strip links an active run to its owning task detail.

## Open

None.
