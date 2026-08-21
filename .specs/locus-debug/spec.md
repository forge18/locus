# locus-debug

**Milestone** M3.5 · **Depends on** `sandbox`, `guardrails`, `marketplace-index`

## Purpose

A DAP client in Rust, because **agents need to debug**. An agent that can inspect a live stack stops
guessing at runtime state from print statements.

**There is no debug UI, and that is a decision.** No breakpoint gutter, no variables pane, no step
buttons — the entire surface is `locus debug` inside a container. You debug in your own editor, on your
own checkout, with the tools you already have; Locus does not compete for that job. What you see of an
agent's debugging is what it reports.

Two things follow: the client is a fraction of the cost of an extension host, because a UI is most of
what makes a debugger expensive; and the design does not carry a half-built pane waiting for someone to
want it.

## Governed by

- PLAN.md §Agents need real tools — and it settles the DAP question
- PLAN.md §Debugging — the session is the core's, the CLI is stateless
- PLAN.md §Editor — no debug UI, restated so it is not rediscovered

## Contract

```
locus debug start [--config NAME]     launch under the adapter
locus debug break FILE:LINE [--if EXPR] [--log FMT]
locus debug run|step|next|finish|continue
locus debug stack|vars [--frame N]|eval EXPR
locus debug stop
```

**The session lives in the core, not the CLI.** A breakpoint set by one command has to still exist when
the next one runs, and each `locus debug` invocation is a separate process that exits. So the session is
keyed by run id, the adapter process is long-lived inside the agent's container, and **every command is
a request against it. The CLI holds nothing.**

**Five things this has to get right, each a way agents lose time:**

- **Logpoints before breakpoints.** `--log` prints and keeps running; `break` stops the world. An agent
  that stops the world then has to remember to continue it, and **a stopped process nobody resumes looks
  exactly like a hung run**. Logpoints are the default advice in the tool's own docs blob.
- **A paused program is not an idle agent.** The idle guardrail counts events on the run's stream; a
  debug session parked at a breakpoint **suppresses it**, because the agent is working and the program
  is not. Without this every real debugging session trips a guardrail at 60 seconds.
- **`--config` comes from project settings**, the same place the run script lives. Debugging is not a
  different way to start the app; it is the same command under an adapter.
- **Adapters are tools.** `codelldb`, `debugpy`, `js-debug` are marketplace entries in the agent's
  allowlist — in the image or not available. Same rule as every other tool, and the reason `locus debug`
  has honest coverage limits rather than pretending.
- **The adapter dies with the run.** No cleanup path, because the container takes it.

## Acceptance

1. A breakpoint set by one CLI invocation still exists for the next — proving the session is core-held.
2. The CLI process holds no state between invocations.
3. `--log` prints and continues; `break` stops. Both are observable in the run's events.
4. A run parked at a breakpoint for five minutes trips **no** idle guardrail.
5. `--config` resolves from project settings, and debugging uses the same run command under an adapter.
6. An agent without an adapter in its allowlist gets an honest "not available", not a silent failure.
7. Killing the run takes the adapter with it, with no orphan process.
8. `vars` and `eval` return structured JSON.
9. **No debug UI exists** — no gutter, no variables pane, no step control anywhere in the app.

## Open

- Adapter coverage is the standing risk PLAN.md names: the client is one implementation, but every
  language needs its own adapter baked per project. Node, Python and Rust are well served; the tail is
  not, and `locus debug` is only as broad as the adapter set.
