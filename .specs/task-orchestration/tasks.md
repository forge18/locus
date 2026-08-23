# task-orchestration — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Persist a task's selected workflow and root-session identity | — | `cargo test -p locus-core task::workflow_session_link` |
| 2 | Refuse an orchestration root session without an owning task | 1 | `cargo test -p locus-core task::root_session_requires_task` |
| 3 | Project-eligible workflow selection with a confirmable default | 1 | `cargo test -p locus-core task::workflow_selection` |
| 4 | Create a manual task draft through the task command API | 1,3 | `cargo test -p locus-cli task::create_manual` |
| 5 | Start a task by creating its root workflow execution and session | 1,4 | `cargo test -p locus-core task::start_creates_root_session` |
| 6 | Link loop-reset runs to the same task through the root session | 5 | `cargo test -p locus-core task::reset_stays_with_task` |
| 7 | Link invoked child-agent runs to the owning task | 5 | `cargo test -p locus-core task::child_runs_stay_with_task` |
| 8 | Return a task-scoped run tree, workflow, evidence, and external-link summary | 5,7 | `cargo test -p locus-core task::detail_summary` |
| 9 | Scope pause, cancel, handoff, and attention actions to an owned task run tree | 8 | `cargo test -p locus-core task::controls_are_task_scoped` |
| 10 | Render the project Kanban from task rows only | 4 | `pnpm -C apps/desktop test -- automate/kanban-tasks` |
| 11 | Render a project List from the same task query as Kanban | 10 | `pnpm -C apps/desktop test -- automate/list-tasks` |
| 12 | Assert Kanban and List resolve each card or row to the same task locator | 10,11 | `pnpm -C apps/desktop test -- automate/task-view-parity` |
| 13 | Render manual task creation from Kanban | 3,10 | `pnpm -C apps/desktop test -- automate/create-task-kanban` |
| 14 | Render manual task creation from List | 3,11 | `pnpm -C apps/desktop test -- automate/create-task-list` |
| 15 | Assert both entry points produce one task draft contract | 13,14 | `pnpm -C apps/desktop test -- automate/create-task-parity` |
| 16 | Render task detail with workflow, root session, child runs, evidence, and controls | 8,12 | `pnpm -C apps/desktop test -- automate/task-detail` |
| 17 | Assert agents are not a peer-level Automate list | 10,11 | `pnpm -C apps/desktop test -- automate/no-agent-list` |
| 18 | Link the running strip to the owning task detail | 8,16 | `pnpm -C apps/desktop test -- shell/running-strip-task-link` |
