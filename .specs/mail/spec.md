# mail

**Milestone** M3 · **Depends on** `store`, `event-store` · **Blocks** `handoffs`, `guardrails`

## Purpose

Agent-to-agent messages, Rust-native, identical for every harness. Also the system **you** are a
participant in — PLAN.md's inbox is not a separate mechanism, it is you as an addressee in the same
mail system agents use.

`amq` is inspiration only, read for its verb set and scoping model. It stores data as files, which is
precisely what putting mail in Postgres is meant to fix.

## Governed by

- PLAN.md §Shared services — one Rust implementation, every harness
- PLAN.md §The user inbox — silence is the default
- PLAN.md §Workflow guardrails — waiting ≠ idle
- PLAN.md §Event sourcing and its two carve-outs — threads and delivery state are folded

## Contract

```
locus mail send|list|read|reply|drain|wait
```

**`wait` blocks with a 15-minute default timeout, then returns empty.** Critically, **it sets
`waiting`** — so it never reads as idle. One mechanism, four callers: `locus ask`, `locus mail wait`, a
debug session at a breakpoint, and a `Gate`. Without it every deliberate block reads as a stall and
trips the idle guardrail.

**The inbox is you in the same system.** `locus ask` from an agent, a `Gate` waiting on approval, a
guardrail trip, a contradiction found at ingest, a workflow goal awaiting sign-off, a finished run
needing review — all of it is a message addressed to you, threaded, with the session it came from.

**Silence is the default.** A session working normally produces nothing. The inbox tells you when
something *needs you*, not when something *happened*.

**Every item resolves to something.** A `Gate` opens the artifact it waits on, a `locus ask` opens that
session's chat, a contradiction opens both wiki pages. **An item that only reports that something
happened is a notification, not inbox work** — and does not belong here.

**Threads and delivery state are a fold**, with no carve-out: `mail.sent`, `mail.read`, `mail.drained`
append, and the inbox is the projection. `drain` returning everything pending and leaving the thread
empty is therefore a property of replay, not a mutation that could half-apply — which matters because
`drain` is the one verb that would otherwise lose messages if it failed midway.

**A handoff is not mail.** Mail is a message between agents that both keep working; a handoff transfers
ownership and does not come back.

## Acceptance

1. Two agents in different containers exchange a threaded message.
2. `wait` sets the run's state to `waiting` with a reason, and the idle guardrail does not fire while
   it is set.
3. `wait` returns empty after its timeout rather than hanging.
4. `drain` returns everything pending and leaves the thread empty.
5. `locus ask` reaches the human inbox with its session attached, and blocks.
6. Threads, messages and delivery state are projections; nothing writes them directly.
7. `locus rebuild --schema mail` reproduces every thread and its delivery state from the log alone.
8. A `drain` interrupted mid-way leaves the thread either fully drained or untouched, never partial.
6. Every inbox item carries a locator that resolves to the thing it is about.
7. An item with no resolvable target is rejected as a notification, not stored as inbox work.
8. A session running normally produces zero inbox items.
9. Mail survives a harness swap mid-project.

## Open

- Whether an agent can address the human directly with `locus mail send`, or only through `locus ask`.
  PLAN.md gives `ask` as the escalation verb and describes the inbox as the same mail system, which
  leaves the direct path ambiguous.
