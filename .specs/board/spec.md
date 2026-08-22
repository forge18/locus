# board

**Milestone** M5 · **Depends on** `store`, `event-store`, `workflow-engine`, `screens-automate`

## Purpose

Deliberately small. Fixed columns across every project — not configurable — and **only the gating that
something else already depends on**.

## Governed by

- PLAN.md §The board — the columns, the task shape, the two gating rules
- PLAN.md §Teams — dependency edges come from the workflow graph
- PLAN.md §Event sourcing and its two carve-outs — the board is a projection, not a table you write

## Contract

```
Ready → In Progress → Testing → Reviewing → Waiting For Approval → Done
```

Four of the six map onto machinery already in the plan: **In Progress** is a run, **Testing** is the
task's verify command, **Reviewing** is a reviewer agent or a `Gate`, and **Waiting For Approval** is a
human decision — **which means it is an inbox item, not a place to go looking.**

**Blocked is a status, not a column.** It shows as an icon and is orthogonal to progress, because a task
can be blocked at any point on the line, not only before it starts. Dependency auto-unblock clears the
*status* when a predecessor completes; **it never moves the card.**

```
task
  summary · description
  column · blocked (with the reason and what would clear it)
  assigned_agent        nullable — unassigned is normal
  project · repo · session
  blocked_by[]          generated from the workflow graph, not hand-drawn
  verify                the runnable check
  evidence[]            run + the events that justify a transition
  github_issue          nullable, linked by explicit action in either direction
```

**A task is a fold, not a row.** Nothing writes `board.tasks`. A move appends `task.moved`, an
assignment appends `task.assigned`, and the card you see is the projection. The consequence worth
having is that **a status disagreeing with its evidence stops being possible rather than merely
detectable** — the status *is* the evidence, replayed. The two gating rules below are properties of the
fold, so an agent cannot route around them by writing the table; there is no table to write.

**Two gating rules, and no more:**
- An agent cannot move a card to **Done** without evidence.
- **Blocked** clears automatically and never manually — it is derived from `blocked_by`, so clearing it
  by hand would just be lying about a dependency.

**Everything else is unrestricted.** You can drag anything anywhere; the constraints exist to stop an
agent asserting completion, not to stop you working. A drag appends `task.moved` with `actor: human`,
which is also how the board answers *"who moved this, and when"* without a separate audit table.

**Evidence proves the requirement was met, not that the feature is right.** No amount of verification
reaches outside its requirement — which is why contract completeness is the highest-leverage unsolved
problem in a system shaped like this, and why the planning module's *elicitation* is where the quality
actually comes from.

## Acceptance

1. Six columns exist and no API or UI path adds, removes, or renames one.
2. `blocked` is a status: a blocked card stays in its column, and a test blocks a card in each of the six.
3. An agent moving a card to Done **without evidence is refused**; a human can still do it.
4. Completing a predecessor clears the dependent's blocked status **without moving it**.
5. `blocked_by` edges are generated from the workflow graph, and there is no hand-drawing path.
6. Manually clearing `blocked` is refused.
7. A human can drag any card to any column.
8. Evidence links resolve to a run and the specific events that justify the transition.
9. A card in Waiting For Approval appears in the inbox.
10. No code path writes `board.tasks` directly — a test asserts the projector is the only writer.
11. `locus rebuild --schema board` reproduces every card byte-identically from the log alone.
12. `locus rebuild --schema board --to <stream_pos>` shows a task in the column it was in then.
13. A human drag and an agent move are the same entry kind, distinguished only by `actor`.

## Open

- The handoff's Kanban draws column 2 as **Building**; PLAN.md names it **In Progress**. One label wins,
  and it is decided here since this is where the column becomes real.
