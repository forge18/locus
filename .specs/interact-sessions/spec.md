# interact-sessions

> **Retired:** `planning-workspace` explicitly retires the Interact route and all disposable board-less session, branch, research, commit, promote, and discard capabilities. This file is preserved as the historical implementation contract only.

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision`, `agent-interface` · **Blocks** M2 workspace surfaces

## Purpose

Interact is where you talk to an agent without first putting anything on the board. A session here is
"a container, a branch and an agent you talk to directly" — nothing more. It carries no card, no plan
and no gate until you deliberately promote it, and most sessions never are: they exist to let you try
something, read code, or run a command without turning that into a tracked obligation.

This spec is the screen: the sessions rail, the embedded agent panel, the "Changed this session" rail,
and the two ways a session's work leaves the sandbox. It also settles the backend model those panes
render — a session with no board task is a first-class, named state rather than a null `board_task_id`
quietly meaning something the rest of the system has to special-case.

## Governed by

- PLAN.md §What a session is — the session / run / turn model; this feature adds no new entity, only a
  state a session can be in while its `board_task_id` is unset
- PLAN.md §The git model — a local remote, not shared worktrees — what pushing `interact/<id>` and
  deleting a branch mean
- PLAN.md §Adding a repo — Locus never works in `main`/`master`; every session branches
- `.specs/agent-interface/spec.md` §Contract — the embedded agent panel this screen renders in its
  center pane; this spec does not restate that contract, only the two settings Interact overrides
- `.specs/run-supervisor/spec.md` §Contract — the session/run boundary that Discard's container kill
  and reconciliation ride on
- `.specs/repo-manager/spec.md` §Contract — the bare local remote and merge-back path that Commit to
  branch and Discard operate against
- `docs/UI_MOCKUP_REVIEW.md` — "Interact (new surface)" section, "The agent panel" section, and the
  Merge modal paragraph under "Navigation"

## Contract

### Sessions rail

246px, collapsible to a 40px dot strip that preserves which session is live and which is selected.
Each card carries: a live dot or state icon, name, harness, age, and a meta chip — `2 changed`,
`clean`, `→ #1184`, or `discarded`. Each row has a delete control that acts with **no confirmation**.

Footer, verbatim: "A session is yours alone — no card, no plan, no gate. Nothing here reaches the
board unless you promote it."

Empty state, verbatim: "A session is a container, a branch and an agent you talk to directly. Start
one to try something without putting it on the board."

The rail's per-row delete and the center panel's Discard action (below) are the same operation reached
two ways: destroying container and branch with no confirmation is the row control's whole point, so it
cannot be a softer path than the panel's own Discard.

### The session contract

A session opened from Interact is an ordinary `agents.sessions` row (`run-supervisor`'s session/run
model, unchanged) with `board_task_id` unset and a branch named `interact/<id>` instead of
`agent/<run-id>`. It behaves exactly as `run-supervisor` describes — resumable across runs, cost
summed across them — with one addition: it also carries an **interact state**, described below, that
answers the one question the board-oriented model never had to ask: what does a session with nothing
on the board mean for gating?

### Session states

| State | Meaning | Gating |
| --- | --- | --- |
| `open` | live or paused, `board_task_id` is unset | no card, no plan, no gate; nothing in the Inbox; this screen is the only account of what it touched |
| `promoted` | attached to a board task, `board_task_id` is set | the diff now takes the normal board gate; Interact stops offering any action over it |
| `discarded` | container killed, `interact/<id>` deleted from the bare remote | terminal; the session row and its transcript are retained and readable |

A session starts `open` and can move to `promoted` or `discarded` exactly once; neither transition
reverses. Promoting an already-promoted or discarded session is refused. Discarding a promoted session
is refused — once a session is on the board, ending it goes through the board's own gate, not through
Interact.

### The embedded agent panel

The center pane is the agent panel from `.specs/agent-interface/spec.md`, unmodified except for two
settings this screen pins rather than leaves to the panel's own defaults:

- **Token/cost is shown**, overriding the panel's off-by-default setting — a session with no card is
  the one place cost is the only accounting that exists for it.
- **The research toggle is wired**, and research **shares space with the "Changed this session" rail**
  rather than opening beside it: opening research hides the rail, and closing research restores it.
  The two never render together.

Every other panel control — thinking, tool calls, elicitation, checkpoints, plan tray, run-config chip
— is exactly what `agent-interface` specifies and is not restated here.

### Changed this session

320px, replaced by the research pane while research is open (above). Header: repo, base commit,
branch (`interact/<id>`). Body: one row per changed file — change marker, name, path, diff stat — and
a file count.

Empty state, verbatim: "Nothing written yet — this session has only read and run commands."

One state-dependent note, verbatim per state:

| State | Note |
| --- | --- |
| open | "This session has no card, so no approval gate and nothing in your Inbox. This panel is the only account of what it touched." |
| promoted | "This session was promoted to a card, so its diff now takes the normal gate. What you see here is the record of what it touched before that." |
| discarded | "This session was discarded. The container and branch are gone; the transcript stays for the record." |

### Ending a session

Two actions, offered only while the session is `open` — a `promoted` session ends through the board's
gate, and a `discarded` session has nothing left to end:

- **Commit to branch** — "Pushes `interact/<id>` to `<repo>`. You land it yourself, later." Its caret
  opens the shared merge modal (`docs/UI_MOCKUP_REVIEW.md` §Navigation, Merge modal paragraph), naming
  `interact/<id>` and the target repo. Pushing does not change the session's state, promote it, or kill
  its container — the session can keep running afterward.
- **Discard** — "Kills the container and deletes the branch. The transcript stays." Irreversible; moves
  the session to `discarded`.

### Backend consequences

- A session with no board task is the `open` interact state, not a null `board_task_id` the rest of
  the system has to interpret. Every reader of `agents.sessions` sees a real state, not an absence.
- **Promotion** attaches an existing or newly created board task, setting `board_task_id` and moving
  `open` → `promoted`. From that instant the session's diff is reachable only through the normal board
  gate (`.specs/board/spec.md` §Contract, once built) — Interact offers no merge, commit, or discard
  action over a promoted session.
- **Discard** destroys the container and deletes `interact/<id>` from the bare local remote
  (`.specs/repo-manager/spec.md` §Contract) in the same step. The session row, its events, and its
  transcript are retained — nothing about the session's history disappears, only its live workspace.
- Boot-time reconciliation (`run-supervisor`'s "every start reconciles") must not treat a discarded
  session's missing container as a crash: a discarded session is expected to have no container, and
  reconciliation skips it rather than filing an aborted-run inbox item.

## Supersedes

This spec supersedes nothing — it is new. The surface it occupies is the project's hands-on git-review
surface from the historical desktop iteration:
screen. Interact renames that slot per `design-revision` and rebuilds it around board-less sessions
that talk to an agent directly, rather than a screen that assumes a task and a branch already exist.

## Acceptance

1. A session opened from Interact starts `open`, carries no `board_task_id`, produces no card, no
   plan, and no gate, and generates no Inbox item.
2. Promoting an `open` session sets `board_task_id` and moves it to `promoted`; promoting a `promoted`
   or `discarded` session is refused.
3. Once `promoted`, a session's diff is reachable only through the board's own gate — no Interact
   action (Commit to branch, Discard, or the rail's delete) is offered over it.
4. Discarding an `open` session kills its container, deletes `interact/<id>` from the bare remote, and
   moves it to `discarded`; discarding a `promoted` or already-`discarded` session is refused.
5. A discarded session's row, events, and transcript remain readable after its container and branch
   are gone.
6. Commit to branch pushes the current `interact/<id>` HEAD and changes no session state; the session
   can still be worked in, promoted, or discarded afterward.
7. The sessions rail collapses to 40px and restores to 246px without losing which session is live or
   which is selected.
8. The meta chip renders `<n> changed` for a dirty `open` session, `clean` for a clean `open` one,
   `→ <task>` once `promoted`, and `discarded` once `discarded`.
9. The rail's per-row delete acts immediately, with no confirmation dialog, and performs the same
   destroy path as the panel's Discard action.
10. The rail footer, the rail empty state, and the "Changed this session" empty state render their
    three strings verbatim.
11. The three state-dependent notes in "Changed this session" render verbatim for their respective
    state, and only one renders at a time.
12. Opening the agent panel's research pane hides "Changed this session," and closing research
    restores it; the two never render together.
13. Commit to branch's caret opens the shared merge modal naming `interact/<id>` and the session's
    repo.
14. The embedded panel shows token/cost by default in this screen, overriding `agent-interface`'s
    off-by-default setting; every other panel control is unchanged from that spec.

## Open

- Where the promote action is triggered from is undrawn — the mockup shows the resulting `→ #1184`
  meta chip but not the control that gets a session there. This spec fixes the transition's backend
  contract (state 3, above) and leaves the trigger's exact placement (rail row, panel header, or both)
  to implementation.
- The merge modal's "Evidence travelling with it" column assumes a card: a verify command, plan
  clauses satisfied, an analyzer result. An `open` session committed via Interact has none of those —
  whether the modal degrades gracefully for a card-less commit or Commit to branch renders a reduced
  variant of it is unresolved.
- Whether a `promoted` session's container keeps running under Interact or its lifecycle transfers
  entirely to whatever supervises the board task from that point on.
