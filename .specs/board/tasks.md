# board — tasks

Carried from `screens-automate`: column 2 is "Building" in the handoff and "In Progress" in PLAN.md.
Task 1 settles it and updates whichever document loses.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Column enum, fixed at six; settle the column-2 label in both documents | — | `cargo test -p locus-core board::six_fixed_columns` |
| 2 | Assert no path adds, removes or renames a column | 1 | `cargo test -p locus-core board::columns_are_closed` |
| 3 | Task table with every field in the contract | 1 | `cargo test -p locus-core board::task_shape` |
| 4 | `blocked` as a status orthogonal to column | 3 | `cargo test -p locus-core board::blocked_is_a_status` |
| 5 | Block a card in each of the six columns | 4 | `cargo test -p locus-core board::blockable_anywhere` |
| 6 | Transitions with an actor, agent or human | 3 | `cargo test -p locus-core board::transitions` |
| 7 | Evidence links: a run plus the events justifying the move | 6 | `cargo test -p locus-core board::evidence_links` |
| 8 | Refuse an agent moving to Done without evidence | 7 | `cargo test -p locus-core board::agent_done_needs_evidence` |
| 9 | Allow a human to move to Done without evidence | 8 | `cargo test -p locus-core board::human_is_unrestricted` |
| 10 | `blocked_by` edges generated from the workflow graph | 3 | `cargo test -p locus-core board::edges_from_graph` |
| 11 | Assert there is no hand-drawing path for edges | 10 | `cargo test -p locus-core board::no_manual_edges` |
| 12 | Auto-unblock on predecessor completion | 10 | `cargo test -p locus-core board::auto_unblock` |
| 13 | Auto-unblock clears the status and never moves the card | 12 | `cargo test -p locus-core board::unblock_does_not_move` |
| 14 | Refuse a manual `blocked` clear | 12 | `cargo test -p locus-core board::no_manual_unblock` |
| 15 | A waiting agent picks up an auto-unblocked task without a human | 12 | `cargo test -p locus-core board::picked_up_automatically` |
| 16 | Waiting For Approval cards appear in the inbox | 3 | `cargo test -p locus-core board::approval_is_an_inbox_item` |
| 17 | `locus task list\|show\|move\|assign\|comment` | 3 | `cargo test -p locus-cli task::verbs` |
| 18 | Wire the Kanban screen to real tasks with drag | 17 | `pnpm -C apps/desktop test -- kanban/from-core` |
