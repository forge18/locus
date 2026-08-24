# manage-revision

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision`, `plan-revision` · **Blocks** M5 project management

## Purpose

Manage is the current mockup's name for the task board — Kanban and List replace the prior task surface, plus
two new views, Graph and Timeline, over one toolbar. `task-orchestration` already settled that Manage
is task-centric: a task owns its workflow execution and root session, and there is no peer-level
Agents list. This spec carries that constraint into four views instead of two, states where Graph and
Timeline read their data from, and settles what the four views show — not the fold or the gating rules
underneath them, which stay where they are.

The screen-by-screen contract lives in
[`docs/UI_MOCKUP_REVIEW.md`](../../docs/UI_MOCKUP_REVIEW.md) § Manage; this spec does not restate it.

## Governed by

- `PLAN.md` §The board — the six columns, blocked as a status, the two gating rules
- `PLAN.md` §Teams — the workflow *is* the team — dependency edges come from the workflow graph, which
  is what Graph draws
- `PLAN.md` §Workflow guardrails — stuck detection and kill-and-reassign, which List's guardrail banner
  surfaces
- `PLAN.md` §Handoffs — a payload for a mechanism that already exists — the payload List's Hand off
  control drafts
- `docs/UI_MOCKUP_REVIEW.md` — the Manage section, all four views
- `.specs/board/spec.md` — the fold, the task shape, the two gating rules this feature only renders
- `.specs/task-orchestration/spec.md` — task-centric Manage, the root-session/session-fan-out
  invariant, no peer-level Agents list
- `.specs/handoffs/spec.md`, `.specs/guardrails/spec.md` — the stuck / kill-and-reassign machinery the
  List view surfaces

## Contract

One toolbar governs all four views: segmented **Kanban / List / Graph / Timeline**, plus **Import
task** and **Add task**. Both actions open the shared task draft `task-orchestration` already defines
— confirmed workflow before start — regardless of which view they were opened from.

### Kanban

Six columns, each with a count: **Ready · In Progress · Testing · Reviewing · Pending Approval ·
Done**. A **Hide Done** toggle. Header meta: "n cards · 3 in flight per person."

Card decoration, all sourced from the task fold and its evidence:

- **Live pulse** — workflow name and running token count, shown while the task's root session is
  active.
- **Blocked marker** — names the gate holding the task, derived from `blocked_by` per the board fold,
  never hand-set.
- **Stuck ring** — `stuck 3/3`, sourced from the guardrail's stuck-iteration count.
- **Testing** cards show the task's `verify_command`.
- **Reviewing** cards show "Gate: reviewer agent" (or the named `Gate` node).
- **Done** cards show the evidence summary that satisfied the gating rule.

**Footer analytics** — median dwell per column as a bar chart, computed from `board.task_transitions`,
with the reading: "The two slowest columns are the two that need a human. Agents are not the
constraint here — the median card spends thirty-eight minutes being built and seventeen hours waiting
to be looked at."

### List

Left: **Active** / **Inactive**, each with a count, then session cards — status dot, agent, role,
tokens, task, workflow and step, status chip, tool, run count. Footer: "Sorted by needs-attention, then
activity. Selecting one does not close the others — a session you stopped watching is not a session
you ended."

Right, for the selected session:

- **Live** — an iteration bar `n/max_iterations` (guardrails default 8), tool errors against baseline,
  token burn, last file write, and a verdict.
- **Closed** — the outcome and "Nothing is running — this is the record, not a live stream." No metric
  in this state may read as live.
- A transcript pane, either state.
- A **stuck guardrail banner** when the guardrail's three-stuck-iteration trigger has fired: "Guardrail
  — kill & reassign after 3 stuck iterations", with the handoff payload's counts — done, remaining,
  attempted, open — read straight from the drafted `handoffs` artifact, never recomputed. **Hand off to
  <agent>** ends the session and opens the successor's per the `handoffs` contract; **Let it run**
  dismisses the banner without ending the session.

Kind vocabulary: `run` (violet, pulsing), `wait` (ring), `idle` (dim), `bad` (red). No fifth value.

### Graph (new)

A dependency DAG. Node = task card (title, status line). Edges come one-to-one from
`board.task_dependencies` — every row is an edge, and there is no edge with no row. Edge color is
load-bearing: grey for an ordinary dependency, amber for an edge into or out of a task **blocked on an
approval the viewer owes**. Caption: "Left to right is dependency depth, not time."

Right rail **Unblocks most** — the same ranking `PriorityMethod::UnblocksMost` produces in
`crates/locus-core/src/runtime/dispatch.rs` (the dispatch priority method; see `dispatch-revision`),
not a second computation over the same graph. Reading: "Two of the four cards holding up the most work
are waiting on a human, not an agent — the same story the dwell chart tells."

### Timeline (new)

Swimlanes grouped by workflow, a seven-day axis. One bar per card, split into segments per board
column, with the time-in-column value shown per segment. Every segment and every time-in-column value
derives from `board.task_transitions` — a task's transitions in strict order give both the column
sequence and each segment's duration, and the two must never disagree with what Kanban's dwell chart
computes from the same table. Legend: Ready, In Progress, Testing, Reviewing, Pending Approval, and
"stuck or blocked." Caption: "Bar length is wall-clock, not agent time. The widest bars are almost
entirely amber and slate — a card is on the board far longer than any agent is working on it."

### Task-centric invariant, carried forward

None of the four views renders agents or sessions as a primary row or card outside List, and List's
sessions are reached through their owning task, never as a peer-level Agents list. Pause, cancel,
handoff, guardrail, and needs-attention controls apply to a task's owned run tree, exactly as
`task-orchestration` already states — Graph and Timeline add no new control surface of their own.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `board` — the Kanban surface (columns, counts, Hide Done, card decoration, dwell footer) | this spec, for rendering; the six-column fold, blocked-as-status, and the two gating rules stay in `board` |
| `task-orchestration` — the Kanban/List surface and the shared task-draft creation flow | this spec, for all four views and the toolbar; the task-centric contract, root-session ownership, and control-scoping stay in `task-orchestration` |

Both superseded specs keep the parts this feature does not touch — the fold, the gating rules, and the
task/session/run ownership model. Only how Manage renders them moves here.

## Acceptance

1. All four views render from the task fold and `board.*` tables; no view holds fixture-only data.
2. Kanban shows six columns with counts, a Hide Done toggle, and every card decoration named above,
   each traceable to the fold or the guardrail state — none hand-set.
3. The Kanban footer dwell chart and its "two slowest columns" reading are computed from
   `board.task_transitions`, not asserted as static copy.
4. Graph's edges resolve one-to-one to `board.task_dependencies` rows; an edge is amber only when it
   touches a task blocked on an approval the viewer owes, grey otherwise.
5. Timeline's per-column segment durations sum to each card's total time-in-column, both computed from
   `board.task_transitions`, and the two never disagree.
6. Graph's "Unblocks most" ranking and `PriorityMethod::UnblocksMost` order the same task set
   identically — asserted against the same input, not independently re-derived.
7. List's stuck guardrail banner fires only on the guardrail's own three-stuck-iteration trigger; Hand
   off drafts a handoff carrying `done[]`, `remaining[]`, `attempted[]`, `open[]` counts per the
   `handoffs` contract, and Let it run dismisses the banner without ending the session.
8. List's Kind vocabulary is exactly `run`, `wait`, `idle`, `bad` — no fifth value appears in fixture
   or code.
9. Live sessions in List show iteration `n/max_iterations`, tool errors against baseline, token burn,
   and last file write; closed sessions show only the outcome and the record copy — never a metric
   that reads as live.
10. No view under Manage renders a peer-level Agents list; every session, workflow execution, and
    agent run is reached through its owning task.
11. Import task and Add task, invoked from any of the four views, produce the same task draft and the
    same audit entries `task-orchestration` already defines.
12. Selecting a card or row in any of the four views resolves to the same
    `locus://<project>/task/<id>` locator.

## Open

- **Column-name conflict.** The mockup's Kanban draws the fifth column as **Pending Approval**;
  `PLAN.md` §The board and `.specs/board/spec.md` name it **Waiting For Approval**. `board/spec.md`
  already carries one such conflict open for column 2 (`Building` vs `In Progress`) and settles it
  there; this is a second, separate conflict on column 5, and it is not resolved here — one label
  wins where the column becomes real, in `board`.
- **`unblocks-most`'s home.** The ranking currently lives in `crates/locus-core/src/runtime/dispatch.rs`
  as `PriorityMethod::UnblocksMost`. Whether Graph's right rail calls that function directly or a
  `manage`-scoped projection re-reads its output is undecided — resolved in `dispatch-revision`, which
  does not exist yet.
