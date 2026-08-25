# tool-compaction

**Milestone** M3 · **Depends on** `materializers`, `artifacts`, `telemetry`

## Purpose

The cheapest token is the one that never enters context. `local-dx` puts this in front of the tool
rather than behind it: rewrite verbose commands and compact their output *before* the result is
appended, reported at 60-90% savings on ordinary development operations.

Locus already materializes hooks into every container, so **the same interception ships as a
`PreToolUse` hook in the base image** — one implementation, every harness, needing no cooperation from
the agent.

## Governed by

- PLAN.md §Token discipline #2 — prevention at the tool boundary
- PLAN.md §Token discipline #3 — diagnosis as a query
- PLAN.md §Artifacts — where over-threshold results go

## Contract

A `PreToolUse` hook in `locus/base-<harness>`, materialized into every run.

**Two behaviors:**

1. **Rewrite** a verbose command into its compact equivalent before it runs.
2. **Compact** the result before it is appended to context.

**Over-threshold results become artifacts.** Anything past the threshold is written as a `payload`
artifact with **a one-line summary and an id** left in its place; `locus artifact get` fetches the body
if it turns out to matter. This is the fourth surface the summary-with-a-handle rule applies to, and the
one that catches everything the other three do not.

**The rule it enforces:** a tool high on bytes but low on calls is returning too much per call. Narrow
the read, add a line range, compact the output.

**Diagnosis is a query, not a tool.** Every `tool_result` is already a normalized row, so ranking tools
by result payload is a `GROUP BY` — per agent, per project, per harness. **Halving the biggest
contributor beats eliminating three small ones**, which is what makes "this agent is expensive"
actionable instead of merely true.

**Hooks log and inject; they never think.** This one fires on every tool call, so it must be cheap: no
model call, no synchronous socket write, and the same 100ms discipline the memory injection path carries.

## Acceptance

1. The hook materializes into the first-party Pi harness and trusted user harness plugins and fires on tool calls in each.
2. A verbose command is rewritten before it runs, and the rewrite is visible in the event stream.
3. A large result is compacted before reaching context, and the saving is measured — a test asserts a
   ratio, not merely that something happened.
4. An over-threshold result becomes a `payload` artifact and leaves a summary and an id.
5. The hook never calls a model and never blocks on the socket.
6. The hook exits 0 on every failure path — a broken compactor degrades to no compaction, never to a
   failed tool call.
7. The offender ranking is a query over `tool_result` rows, with no new instrumentation.
8. Turning compaction off changes only cost, never behavior — asserted by comparing event streams.

## Open

- The compaction threshold, shared with `artifacts`. It should be one setting used by both, not two
  that drift.
