# manage-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Pointer lines in `board/spec.md` and `task-orchestration/spec.md` naming this spec as the surface replacement | — | `grep -q "manage-revision" .specs/board/spec.md .specs/task-orchestration/spec.md` |
| 2 | Kanban column counts and Hide Done filter over the task fold | — | `cargo test -p locus-core manage::kanban_counts` |
| 3 | Card decoration: live pulse, blocked marker, stuck ring | 2 | `cargo test -p locus-core manage::card_decoration` |
| 4 | Terminal-column decoration: verify command on Testing, gate name on Reviewing, evidence summary on Done | 2 | `cargo test -p locus-core manage::terminal_column_decoration` |
| 5 | Median dwell per column and the "two slowest need a human" reading, from `task_transitions` | 2 | `cargo test -p locus-core manage::dwell_by_column` |
| 6 | List Active/Inactive session queries and session card fields | — | `cargo test -p locus-core manage::list_sessions` |
| 7 | Live metrics strip: iteration n/max_iterations, tool errors against baseline, token burn, last file write | 6 | `cargo test -p locus-core manage::live_metrics_strip` |
| 8 | Closed-session record: outcome only, no live-shaped field | 6 | `cargo test -p locus-core manage::closed_session_record` |
| 9 | Stuck guardrail banner: trip detection plus handoff payload counts read from the drafted artifact | 6 | `cargo test -p locus-core manage::stuck_banner_payload` |
| 10 | Hand off to `<agent>` action, wired to the `handoffs` contract | 9 | `cargo test -p locus-core manage::handoff_action` |
| 11 | Let it run action dismisses the banner without ending the session | 9 | `cargo test -p locus-core manage::let_it_run` |
| 12 | Graph edges one-to-one from `board.task_dependencies`, grey/amber by approval owed | — | `cargo test -p locus-core manage::graph_edges` |
| 13 | Assert Graph's Unblocks-most ranking matches `PriorityMethod::UnblocksMost` on the same input | 12 | `cargo test -p locus-core manage::unblocks_most_matches_dispatch` |
| 14 | Timeline per-card, per-column segments and time-in-column value from `task_transitions` | — | `cargo test -p locus-core manage::timeline_segments` |
| 15 | Timeline swimlanes grouped by workflow over a seven-day axis | 14 | `cargo test -p locus-core manage::timeline_swimlanes` |
| 16 | Assert Timeline's segment durations and Kanban's dwell chart never disagree on the same task | 5,14 | `cargo test -p locus-core manage::dwell_and_timeline_agree` |
| 17 | Shared toolbar — Kanban/List/Graph/Timeline segmented control plus Import task / Add task | — | `pnpm -C apps/desktop test -- manage/toolbar` |
| 18 | Kanban columns, counts, and Hide Done wired to core | 2,17 | `pnpm -C apps/desktop test -- manage/kanban` |
| 19 | Kanban card decoration wired to core | 3,4,18 | `pnpm -C apps/desktop test -- manage/kanban-card` |
| 20 | Kanban dwell footer chart and reading copy | 5,18 | `pnpm -C apps/desktop test -- manage/kanban-dwell` |
| 21 | List Active/Inactive and session cards wired to core | 6,17 | `pnpm -C apps/desktop test -- manage/list` |
| 22 | List live-session metrics strip and verdict | 7,21 | `pnpm -C apps/desktop test -- manage/list-live` |
| 23 | List closed-session record and transcript pane | 8,21 | `pnpm -C apps/desktop test -- manage/list-closed` |
| 24 | List stuck guardrail banner with Hand off / Let it run | 9,10,11,21 | `pnpm -C apps/desktop test -- manage/list-guardrail` |
| 25 | Graph DAG render — grey/amber edges, "dependency depth" caption | 12,17 | `pnpm -C apps/desktop test -- manage/graph` |
| 26 | Graph Unblocks-most rail with its reading copy | 13,25 | `pnpm -C apps/desktop test -- manage/graph-unblocks-rail` |
| 27 | Timeline swimlane render, legend, "wall-clock" caption | 14,15,17 | `pnpm -C apps/desktop test -- manage/timeline` |
| 28 | Import task / Add task produce one draft from any of the four views | 17,18,21,25,27 | `pnpm -C apps/desktop test -- manage/create-task-parity` |
| 29 | Assert no view under Manage renders a peer-level Agents list | 18,21 | `pnpm -C apps/desktop test -- manage/no-agent-list` |
