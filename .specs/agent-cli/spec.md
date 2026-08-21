# agent-cli

**Milestone** M1 · **Depends on** `store`, `sandbox` · **Blocks** every `locus` verb in every feature

## Purpose

`crates/locus-cli` — the binary agents call from inside their container, and **the MCP replacement**. A
binary costs nothing until invoked; a server sits in context whether used or not.

This spec covers the CLI *itself*: the socket transport, the output contract, and the four verbs that
belong to no other feature. Every domain verb — `memory`, `mail`, `task`, `wiki`, `lsp`, `debug`,
`browse`, `artifact`, `tools`, `handoff`, `lint` — is specified in its own feature and reaches the core
through this client.

## Governed by

- PLAN.md §Process topology — the agent-facing surface is a CLI, not a protocol
- PLAN.md §Agent CLI — the verb list and the `--json` rule
- PLAN.md §Token discipline — output tokens cost roughly five times input
- PLAN.md §Agents are Markdown — the nesting bounds on `agent invoke`

## Contract

**A thin socket client with no logic of its own.** Every verb is a round trip to `locus-core` over
`/run/locus.sock`, so **behaviour cannot drift between what an agent sees and what the app sees.** Logic
that lives in the CLI is logic the UI does not have.

**Verbs owned here:**

```
locus ask <question>              escalate to the human; BLOCKS, and sets `waiting`
locus run status|artifacts        this run's own state
locus agent invoke <name>@<ver>   a nested agent: own container, own clone
locus svc up|down <name>          start a project service; agents get no Docker socket
```

**`locus agent invoke` is bounded in the core at depth 3 and fan-out 4**, which a workflow may lower and
never raise. Depth 3 with fan-out 4 is at most 21 containers, which one machine survives; **depth 4 is
85, which it does not.** The cycle check, depth limit and fan-out cap are not polish — without all
three, one bad graph exhausts the machine.

**Invoke is not a handoff and not mail.** It is a nested run that **returns to its caller**.

**`--json` on every command**, because the caller is usually a model:
- **Compact, never pretty-printed.**
- **Key-packed for uniform tables past a row threshold** — a header row plus value arrays, which runs
  **50-60% smaller than minified JSON** on tabular data.
- Output tokens cost roughly five times input, so this is a cost decision rather than a formatting one.

**The socket is the only channel.** The CLI holds no state, no cache, and no config of its own — a
second source of truth inside the container is exactly what the store exists to prevent.

## Acceptance

1. Every verb is a socket round trip; a test asserts the CLI computes no domain answer locally.
2. `--json` output is compact — a test asserts no pretty-printing on any verb.
3. Key-packing engages past the row threshold and is measured against minified JSON at 50-60% smaller.
4. Below the threshold, output is ordinary JSON — packing a two-row table costs more than it saves.
5. `locus ask` blocks, sets the run's `waiting` state, and reaches the human inbox with its session.
6. `locus run status` and `artifacts` return only this run's own state, never another run's.
7. `locus agent invoke` starts a nested run in its **own container with its own clone**, and returns to
   the caller.
8. Depth 4 is refused; fan-out 5 is refused; a cycle is refused. All three, not one.
9. A workflow can lower the bounds and **cannot raise them**.
10. `locus svc up` starts a service on the project network without the agent touching a Docker socket.
11. A verb the agent's allowlist does not permit fails with a clear message rather than a socket error.

## Open

- The row threshold at which key-packing engages. PLAN.md gives the technique and the saving but not the
  count, and below some size the header row costs more than it saves.
