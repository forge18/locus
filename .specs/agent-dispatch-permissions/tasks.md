# agent-dispatch-permissions — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Persist the per-job bypass-permissions toggle, defaulting to bypass | `desktop-project-operations` | `cargo test -p locus-core dispatch::permission_posture` |
| 2 | Record gated permission requests as waiting human actions, not bypass alarms | 1 | `cargo test -p locus-core dispatch::gated_permission_request` |
| 3 | Render the Dispatch permission control and its per-job consequence | 1 | `pnpm -C apps/desktop test -- dispatch/permission-mode` |
