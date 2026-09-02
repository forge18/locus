# planning-workspace — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Adopt the accepted workspace, navigation, Worker, approval, and capability-policy contract | — | `rg -q 'planning-workspace' .specs/planning-workspace/spec.md` |
| 2 | Move production route authority into a manifest and derive demo inventories from it | 1 | `pnpm -C apps/desktop test -- nav/production-route-manifest` |
| 3 | Replace the shell project selector with page-owned scope controls and preserve non-retired deep links | 2 | `pnpm -C apps/desktop test -- nav/page-owned-scope` |
| 4 | Replace Bots and remove the Interact route from the rail, resolver, and production screen inventory | 2 | `pnpm -C apps/desktop test -- nav/workers-no-interact` |
| 5 | Add durable PlanningWorkspace, WorkspaceSpec, revision, and materialization projections | 1 | `cargo test -p locus-core planning_workspace::schema_contract` |
| 6 | Backfill existing bounded plans as one-spec workspaces without changing their requirement ids | 5 | `cargo test -p locus-core planning_workspace::backfill_is_idempotent` |
| 7 | Add optimistic revision checks, checkpoint freeze, lifecycle transitions, and hard-delete rules | 5 | `cargo test -p locus-core planning_workspace::checkpoint_and_delete_rules` |
| 8 | Link planning sessions, activities, decisions, handoffs, and resumable state through live IPC | 7 | `cargo test -p locus-tauri --lib planning_workspace_commands` |
| 9 | Build the Planning Room for Brief, Shape, Specs, Tasks, Coverage, and Activity | 8 | `pnpm -C apps/desktop test -- plan/planning-room` |
| 10 | Add multi-spec grilling, shared decisions, staleness, and unified dependency validation | 9 | `cargo test -p locus-core planning_workspace::cross_spec_validation` |
| 11 | Freeze approved revisions and materialize all board tasks idempotently with provenance | 10 | `cargo test -p locus-core planning_workspace::approval_materializes_all_tasks_once` |
| 12 | Add page-owned Workers, Telemetry, Knowledge, Manage, Review, Inbox, and Dispatch filters | 3,4,9 | `pnpm -C apps/desktop test -- navigation/page-owned-filters` |
| 13 | Implement non-escalating project/agent/workflow capability resolution and run snapshots | 1,12 | `cargo test -p locus-core capabilities::project_policy_cannot_be_exceeded` |
| 14 | Replace Plan and secondary-surface fixture providers with explicit live and demo providers | 9,12 | `pnpm -C apps/desktop test -- data/planning-workspace-provider-boundary` |
| 15 | Add restart, replacement-agent, approval-retry, scope-isolation, and retired-route acceptance coverage | 6,7,8,11,12,13,14 | `pnpm -C apps/desktop test -- planning-workspace/acceptance` |
