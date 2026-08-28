# bots

**Milestone** M6 · **Depends on** `agent-definitions`, `agent-interface`, `schedules`

## Purpose

Bots are persistent, named agents you talk to directly and hand recurring prompts — the teammate
model, deliberately outside the plan→develop→review loop. A bot is **an agent definition plus one
durable home session**: ad-hoc work is messaging it; ongoing work is a routine, a cron-fired prompt
recorded like any schedule execution. No card, no plan, no gate, no workflow — a bot's only account
of what it did is its conversation and its branch.

This is a consumer of existing machinery, not a new engine: definitions, the session/run model, the
Agent Pane, and the cron scheduler are all reused unchanged. The one lifecycle addition is a **warm
window** — a container lingers briefly after its last activity instead of stopping immediately, so
a follow-up message does not pay cold start.

## Governed by

- PLAN.md §What a session is — the session/run model, reused unchanged; a bot's home session is an
  ordinary session
- PLAN.md §Agents are Markdown — a bot's profile is an agent definition, versioned and immutable
  once referenced
- PLAN.md §The git model — a local remote, not shared worktrees — the `bots/<bot-id>` branch and
  per-run push-back
- `.specs/agent-definitions/spec.md` — the definition contract; the run records the version it used
- `.specs/agent-interface/spec.md` §Contract — the embedded Agent Pane, **unmodified**; this screen
  adds no panel behavior
- `.specs/interact-sessions/spec.md` — the board-less session precedent (no gating, resumable across
  runs) that this surface generalizes; unlike Interact there is no promote and no discard
- `.specs/schedules/spec.md` — cron machinery, recorded results, skip-and-drop overlap; a routine is
  a schedule whose target is a prompt
- `.specs/design-revision/spec.md` §Vocabulary and §Screen inventory — the rail category and view
  this feature adds

**No run-supervisor changes.** The container lifecycle rides the existing stop path; the warm window
adds only an idle timer on top of it.

## Contract

### Rail and screen

Vocabulary gains a tenth rail category: **Bots**, project-scoped. It holds one view, `bots`, bringing
the inventory to thirty views. Locators: `locus://<project>/bots` for the list and
`locus://<project>/bots/<bot-id>` for one bot; the resolver round-trips both.

The screen is two panes: **the bot list on the left, the Agent Pane on the right.** The list is
246px, collapsible to a 40px dot strip, reusing the interact rail's behavior; each row carries a live
dot, name, harness, and last activity. The right pane is `panes/AgentPane.tsx` composed against the
selected bot's home session with **no Bots-specific props** — stream, thinking, elicitation, plan
tray, and controls are exactly what `agent-interface` specifies.

Rail footer, verbatim: "A bot is a named teammate with one conversation and one workspace. It is not
a task and never touches the board."

Empty state, verbatim: "No bots yet. Create one to have a standing agent you can message any time and
hand recurring work to."

### A bot

Creating a bot collects exactly the agent-definitions frontmatter — name, description, harness,
`model_tier`, `task_class`, `tools`, `skills`, `rules`, `memory` — stored as an ordinary definition
plus a `bots` row binding project, definition, and home session. The description is where **rules
that should remain true** live; task-specific instruction belongs in the conversation.

- **One durable conversation.** The home session is an ordinary `agents.sessions` row, resumable
  across runs, cost summed across them. There is no second thread in v1, and routines post into the
  same conversation.
- **One durable workspace.** A branch `bots/<bot-id>` in the project's bare remote (repo-manager
  clone model, unchanged), created on first run, persisting for the bot's life. Work pushes back per
  run; Locus never works in `main`.
- **Latest definition at run start.** The bot resolves its definition's latest version when a run
  starts, and the run records that version. Refining a teammate's standing instructions is normal;
  per-run immutability is already guaranteed by agent-definitions.
- **No promote, no discard.** Bots sit outside the board loop; this surface offers no gate. Getting a
  bot's work toward `main` is a human PR action like any branch.

### Ad-hoc: messaging

Typing into the panel starts the home session if it has no live container — booting one on
`bots/<bot-id>` — and otherwise delivers into the live run. Everything renders through the
unmodified Agent Pane. No card, no plan, no gate, nothing in the Inbox; the conversation and the
branch are the only account of what the bot did.

### Warm window

After the last activity — a message, a stream ending, or a routine run — the container **lingers for
an idle window, then takes the existing stop path.** Default ten minutes, set per project as
`bots.warm_window_minutes`. A warm stop loses nothing: conversation, branch, and memory persist, and
the next message resumes. Boot-time reconciliation treats a warm-stopped bot container the way it
treats any resumable container — expected, never an aborted-run inbox item.

### Routines: ongoing

A routine is `{ bot, prompt, cron, enabled }` — **a schedule whose target is a prompt instead of a
workflow.** Everything schedules already guarantees is inherited and not restated; only the target
differs:

- A firing boots the home session if cold, sends the prompt, and records the execution **with its
  result** — green or red, not "finished".
- **Overlap is skipped, never queued.** A firing while the bot is mid-run is recorded as skipped and
  dropped; the skip count is visible.
- Routine output lands **in the bot's conversation**, attributed as routine-fired, so ad-hoc and
  scheduled history are one readable transcript.
- Pause, resume, edit, and delete keep history, as schedules does.

Routines are managed in a **sheet over the bot view** — list, pause/enable, edit, delete, and a
**Test run** that sends the prompt immediately, marked as a test. A sheet, not a third pane: detail
opens in place, and the screen stays two-pane.

Event triggers — webhook, mail, forge — are out of scope: a routine fires on cron, nothing else.

## Supersedes

Nothing — this is new. It extends `design-revision`'s inventory from twenty-nine views to thirty and
adds one word to its rail vocabulary.

## Acceptance

1. Creating a bot stores a definition plus a `bots` row, and the bot appears in the rail.
2. Messaging a bot with no live container boots one on `bots/<bot-id>` and streams through the
   Agent Pane unmodified.
3. The home session survives a warm stop: the next message resumes the same conversation and
   workspace, with cost summed across runs.
4. The warm window defaults to ten minutes, is set per project, and the timer takes the existing
   container-stop path.
5. Reconciliation treats a warm-stopped bot container as expected — no aborted-run inbox item.
6. A routine fires with the window closed and records its execution with its result.
7. A firing while the bot is mid-run is recorded as skipped and dropped; nothing queues and the skip
   count is visible.
8. Routine output appears in the bot's conversation attributed as routine-fired.
9. Pause, resume, edit, and delete a routine without losing its history.
10. Test run sends the prompt immediately, marked as a test, without changing the routine.
11. A run records the definition version it used; editing the definition changes the next run, never
    a running one.
12. The `bots/<bot-id>` branch persists across runs and never targets `main`.
13. The Bots rail category carries exactly the `bots` view; `resolve(locate(v, p))` equals `(v, p)`
    for both locators.
14. The Agent Pane is unmodified: a bot session renders with no Bots-specific panel code.
15. The routines sheet opens over the bot view; the rail and the panel do not change.

## Open

- Whether the warm window should scale by harness or model — the same shape as guardrails' open
  idle-window question.
- Whether a bot can pin an older definition version instead of resolving latest at run start.
- How work **enters** the loop from a bot: a handoff or promote-like path from `bots/<bot-id>` to a
  board task is deliberately absent in v1 and undrawn.
- Event-triggered routines: which sources, and how narrow a matching rule must be.
