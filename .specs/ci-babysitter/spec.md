# ci-babysitter

**Milestone** M7 · **Depends on** `github`, `guardrails`, `agent-prs`

## Purpose

A failing pipeline pulls its logs, feeds them to an agent, retries a bounded number of times, then
escalates. PLAN.md calls this **the single most-cited reason people leave a run unattended** — both
Sculptor and Agent Orchestrator ship it.

## Governed by

- PLAN.md §M7 — the CI babysitter
- PLAN.md §Workflow guardrails — bounded retries, and escalation as an inbox item

## Contract

On a failing pipeline: fetch the logs, hand them to an agent in a container, let it push a fix, and
retry — **a bounded number of times, then escalate cleanly.**

**Escalation is an inbox item**, carrying what was tried. An agent that gave up after three attempts and
says nothing is worse than one that never started, because the pipeline stays red and nobody is told.

**Retries are bounded by the same guardrail machinery**, not a private counter — `max_iterations` and
kill-and-reassign already exist, and a second retry mechanism would drift from the first.

**The arbiter applies here too.** A pipeline failing for environmental reasons is `noise` and should not
spend the retry budget; a pipeline failing because the change is wrong is a `bug`. Treating both the
same is what makes a babysitter burn its budget on a flaky runner.

## Acceptance

1. A failing pipeline triggers a fetch of its logs.
2. An agent receives the logs and pushes a fix.
3. Retries are bounded, and the bound comes from the guardrail config rather than a private counter.
4. Exhausting the budget escalates as an inbox item carrying **what was tried**.
5. A noise-classified CI failure does **not** spend the retry budget.
6. A deliberately broken build is fixed within budget; a deliberately unfixable one escalates cleanly
   rather than looping.
7. The babysitter never merges — it pushes to the branch.

## Open

- Whether the babysitter runs as an ordinary workflow or as a supervisor behavior. As a workflow it is
  authorable and inspectable; as a supervisor behavior it is always on. PLAN.md does not say.
