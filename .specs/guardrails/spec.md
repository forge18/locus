# guardrails

**Milestone** M3 · **Depends on** `run-supervisor`, `mail` · **Blocks** `handoffs`, `workflow-engine`

## Purpose

What makes leaving a loop unattended defensible. Each guardrail is borrowed from a **measured failure**
in an existing tool, not invented — which is the reason to implement them all rather than the ones that
seem likely.

## Governed by

- PLAN.md §Workflow guardrails — the table and its defaults
- PLAN.md §Token discipline — the accounting is mandatory even when the ceiling is not

## Contract

| Guardrail | Default | Why |
| --- | --- | --- |
| `max_iterations` | 8 | Agents loop endlessly retrying the same broken approach without a hard stop |
| Forced reflection before retry | on | "What failed? What specific change would fix it? Am I repeating myself?" |
| Kill-and-reassign after 3 stuck iterations | on | A fresh context beats a confused one |
| **Waiting ≠ idle** | — | A run carries `waiting` with a reason — `ask`, `mail`, `debug-paused`, `gate` — and idle counts only time outside it |
| Idle detection | 60s | No event on the run's stream for 60s while not blocked. Agent Orchestrator's measured flaw was agents idling 90+s with nobody told |
| Wall-clock ceiling | none | Optional; a loop that cannot finish overnight should stop, not run to morning |
| Token budget | **none — optional** | When set, auto-pause at 85% and notify rather than draining silently |

**Waiting ≠ idle is one mechanism with four callers** — `locus ask`, `locus mail wait`, a debug session
at a breakpoint, and a `Gate`. Without it every deliberate block reads as a stall.

**Idle surfaces once per stretch.** The tile gets an idle icon and a toast fires **once per idle
stretch, never repeatedly** — a guardrail that nags is one people turn off.

**A budget is optional; the accounting is not.** If token usage explains most of the variance in how a
run goes, then **a run that passes verify on four times the tokens is a worse run wearing a green
tick.** Every run carries usage whether or not a ceiling was set, and agent trust is weighted by tokens
per passing run.

**Pause means the loop stops being fed, not that a process is frozen.** The supervisor lets the current
turn finish, holds before the next iteration, and notifies; the container stays up so its state is
inspectable. `SIGSTOP` mid-request would leave sockets half-written and a model call in flight — a
worse problem than the spend it saved.

## Acceptance

1. `max_iterations` stops a loop at its limit and records which guardrail tripped.
2. Reflection is injected before a retry, and its absence is detectable in the event stream.
3. Three stuck iterations produce a **handoff**, not a dropped task.
4. A run blocked on `locus ask` for five minutes trips **no** idle guardrail.
5. A run blocked on `mail wait`, at a breakpoint, or on a `Gate` likewise trips none — all four callers
   asserted, not just one.
6. A genuinely idle run trips at 60s, shows the idle icon, and toasts **once** — a second toast in the
   same stretch is a failure.
7. Every run carries usage even with no budget set.
8. A budget set to N auto-pauses at 0.85N and notifies rather than draining.
9. Pause lets the current turn finish and leaves the container inspectable.
10. A workflow may tighten or relax any default, and the effective value is recorded on the run.

## Open

- Whether the idle window should scale with the agent's `task_class`. A research agent reading for 90
  seconds is not the same as a builder silent for 90 seconds, and 60s is a single number for both.
