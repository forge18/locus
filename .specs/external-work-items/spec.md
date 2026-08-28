# external-work-items

**Milestone** M7 · **Depends on** `task-orchestration`, `forge-providers`, `board` · **Revised by** [IMPACT_EXTERNAL_SYNC](../IMPACT_EXTERNAL_SYNC.md)

## Purpose

Import GitHub issues from a configured work-item plugin into the same local task workflow as manually
created work, then **keep the local board and the external item in sync**: statuses and notes flow in
both directions for linked items. This slice establishes the provider-neutral port; other trackers
remain future plugins. GitHub is the first-party work-item plugin.

The original contract was deliberately one-way — import in, one completion delivery out at Done, no
synchronization in between. That invariant is superseded: a linked external item participates in the
task's life. What does not change is *how* writes land — every sync effect is a board event through
the fold, never a projection write.

## Contract

An `ExternalWorkItemProvider` plugin port normalizes an opaque plugin ID, host/base URL, workspace or
project, native ID, URL, title, body, labels, status, and completion capabilities. A provider plugin
advertises `work_item.snapshot`, `work_item.comment`, and optional `work_item.resolve` capabilities
through the common JSON-RPC executable contract. Its manifest uses `kind = "provider"` with the
`work_item.*` capability namespace. GitHub is the first-party work-item plugin; later trackers are
separate plugins. Core task, workflow, board, and Automate code never matches on a provider name.

Import is explicit. The user chooses a configured provider and item, previews the source snapshot and
workflow, and confirms. Locus creates one local task with the imported snapshot and external identity,
then follows the identical task-owned workflow/session/run model as manual work. Duplicate import of
the same provider identity is refused or opens the existing local task.

### Synchronization capability

Sync is opt-in per provider through optional `work_item.sync` capabilities; a provider without them
keeps the original one-way behavior (import plus the completion delivery below). A sync-capable
provider declares:

- **A status vocabulary and bidirectional mapping** owned by the plugin: each external status maps to
  one of the six board columns (or to none, for statuses with no local meaning), and each column maps
  back to an external representation. For GitHub this is labels plus open/closed; the scheme is the
  plugin's, not core's. `blocked` syncs outward only where the vocabulary declares a representation
  for it; otherwise it stays local.
- **`work_item.pull`** — the changes since a caller-supplied cursor: external status changes and new
  external notes, each with its external timestamp and author.
- **`work_item.push_status`** — apply a normalized column (and blocked, where declared) to the
  external item.
- **`work_item.push_note`** — post a note to the external item with attribution and a
  machine-readable Locus marker in the same HTML-comment form the completion delivery already uses.

Sync runs by bounded polling through the plugin port — no webhooks, no external daemon (per the
PLAN.md shape). Each provider instance carries a poll interval (default one minute) plus an explicit
*Sync now* control; the board view of linked tasks refreshes on window focus. Calls are bounded like
every plugin call.

### What flows, and how

**Statuses, both ways.** A local `task.moved` pushes the new column outward. An external status
change appends `task.moved` with the sync actor and the external change as its evidence — the status
*is* the evidence, replayed, exactly as for human drags and agent moves.

**Notes, both ways.** Every local task note — human and agent alike — posts outward with attribution
and the Locus marker. External notes land in the task's note stream as `Commented` events carrying
their external author and origin. Notes are an append-only merge; there is no note conflict to
resolve. The marker is the loop guard: an inbound note carrying a Locus marker is recorded as
delivered, never re-appended and never re-posted.

**External close is Done with evidence.** When the external item reaches the status its vocabulary
maps to Done, the card moves to Done with the external close as its evidence — the same evidence rule
the Done gate applies to agents, satisfied by the external event rather than a local run. The first
local Done still triggers the completion delivery; when that Done was sync-originated, the outbox
records the delivery as satisfied by the external close instead of posting a duplicate comment, and
resolution of an already-resolved item is recorded, not an error. Reopening afterward is an ordinary
status push; the outbox remains delivered — only the first Done emits a completion.

**Conflicts resolve last-write-wins, visibly.** Status is the only surface that can conflict. When
both sides changed since the last exchange, the newer external or local timestamp wins, and the
decision is not silent: the resulting `task.moved` event's evidence names the winner, both timestamps,
and the reason. Unmapped external statuses are recorded and surfaced, never guessed into a column.

### Unchanged

The completion delivery keeps its shape: on the first transition to local Done, one durable,
idempotent delivery posts a concise completion comment with task locator and evidence, then
resolves/completes the external item when the provider declares the capability; retries deliver only
this completion event. A later edit to *unlinked* source items never creates local state — import
remains the only path in.

## Acceptance

1. Manual and imported tasks use the same task, workflow, root-session, run-tree, and Automate-detail contracts.
2. The import surface lists configured work-item providers and previews an item before creating a task.
3. The first-party GitHub provider plugin imports a normalized issue snapshot through the common port.
4. Duplicate provider identity import opens the existing task and creates no second task.
5. A sync-capable provider declares its status vocabulary and bidirectional mapping; core code matches on no provider name.
6. An external status change moves the local card as a `task.moved` event with the sync actor and the external change as evidence — no path writes the projection directly.
7. A local column move pushes the normalized status outward through the provider's mapping.
8. Local notes — human and agent — post outward with attribution and the Locus marker; external notes land with external attribution as `Commented` events.
9. An inbound note carrying a Locus marker is recorded as delivered and neither re-appended nor re-posted — no synchronization loop.
10. A two-sided status change resolves last-write-wins, and the decision's evidence names the winner, both timestamps, and the reason.
11. An external close moves the card to Done with the close as evidence; a sync move to Done *without* external evidence is refused by the fold, like an agent's.
12. Sync-originated Done records the completion delivery as satisfied without a duplicate comment; resolution of an already-resolved item records, not fails.
13. A provider without sync capabilities keeps the one-way behavior: no pull, no push, completion delivery only.
14. A failed sync exchange retries idempotently from the last cursor and never duplicates outward notes or status writes.
15. A new provider plugin passes the work-item conformance suite, including sync fixtures, without changes to Automate or task orchestration.

## Open

None — the four product decisions (all notes out, plugin-owned mapping, LWW visible, external close
auto-moves with evidence) are settled above.
