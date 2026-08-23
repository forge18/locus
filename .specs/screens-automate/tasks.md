# screens-automate — tasks

> **Historical fixture tasks.** Live task creation, orchestration detail, and external imports are
> specified by `task-orchestration` and `external-work-items`.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Kanban header with fixed-columns and blocked-is-a-status notes | — | `pnpm -C apps/desktop test -- kanban/header` |
| 2 | Project chips and task-view actions | 1 | `pnpm -C apps/desktop test -- kanban/project-chips` |
| 3 | Six-column grid at 9px gaps, labels, and counts | 1 | `pnpm -C apps/desktop test -- kanban/six-columns` |
| 4 | Assert no column add, remove, or reorder affordance | 3 | `pnpm -C apps/desktop test -- kanban/columns-fixed` |
| 5 | Accent Waiting For Approval column head | 3 | `pnpm -C apps/desktop test -- kanban/approval-accent` |
| 6 | Task card with title and metadata | 3 | `pnpm -C apps/desktop test -- kanban/card` |
| 7 | Blocked variant shown in place | 6 | `pnpm -C apps/desktop test -- kanban/blocked-in-place` |
| 8 | Assert a blocked task appears in any column | 7 | `pnpm -C apps/desktop test -- kanban/blocked-is-orthogonal` |
| 9 | Stuck, approval, and done task-card variants | 6 | `pnpm -C apps/desktop test -- kanban/card-variants` |
| 10 | List layout for project tasks | — | `pnpm -C apps/desktop test -- automate/list-layout` |
| 11 | List header, filters, and sort controls | 10 | `pnpm -C apps/desktop test -- automate/list-header` |
| 12 | Task row with workflow, root-session, dependency, and evidence summaries | 10 | `pnpm -C apps/desktop test -- automate/task-row` |
| 13 | Selected, blocked, and stuck row states | 12 | `pnpm -C apps/desktop test -- automate/task-row-states` |
| 14 | Sort task list by needs-attention then activity | 12 | `pnpm -C apps/desktop test -- automate/task-sort-order` |
| 15 | List footer explaining task ownership of runs | 11 | `pnpm -C apps/desktop test -- automate/task-list-footer` |
| 16 | Task-detail header with workflow and root-session locator | 10 | `pnpm -C apps/desktop test -- automate/task-detail-header` |
| 17 | Task detail run tree with agent activity | 16 | `pnpm -C apps/desktop test -- automate/task-run-tree` |
| 18 | Assert task detail renders only canonical event verbs | 17 | `pnpm -C apps/desktop test -- automate/task-canonical-verbs` |
| 19 | Current activity prompt line | 17 | `pnpm -C apps/desktop test -- automate/task-activity` |
| 20 | Task-scoped stuck controls | 16 | `pnpm -C apps/desktop test -- automate/task-stuck-controls` |
| 21 | Task-scoped waiting-for-approval state | 16 | `pnpm -C apps/desktop test -- automate/task-waiting-state` |
| 22 | Assert task conditional states are mutually exclusive | 20,21 | `pnpm -C apps/desktop test -- automate/task-state-exclusive` |
| 23 | Task detail evidence and external-link status | 16 | `pnpm -C apps/desktop test -- automate/task-evidence` |
| 24 | Task selection leaves other task runs active | 12,17 | `pnpm -C apps/desktop test -- automate/task-select-does-not-stop` |
| 25 | Task controls preserve task/run links | 16 | `pnpm -C apps/desktop test -- automate/task-controls-preserve-links` |
| 26 | Running strip opens the owning task detail | 16 | `pnpm -C apps/desktop test -- automate/strip-opens-task` |
| 27 | Visual check for task Kanban, List, and detail | 9,26 | `pnpm -C apps/desktop test -- visual -- board tasks` |
