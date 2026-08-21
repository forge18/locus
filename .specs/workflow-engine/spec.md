# workflow-engine

**Milestone** M4 · **Depends on** `guardrails`, `run-supervisor`, `sandbox`

## Purpose

The execution half. A workflow is **a loop toward a goal**, not a one-shot pipeline — the shape the
field converged on is pick → act → validate → commit → **reset**, iterating with a fresh context each
pass so confusion does not accumulate.

Locus has a real advantage there: the four ad-hoc memory channels that pattern relies on (commit
history, a progress log, a task file, a context file) are one queryable, scoped store here.

## Governed by

- PLAN.md §The Workflow Canvas — loop semantics, `Condition`, where `Verify` runs
- PLAN.md §When `Verify` fails, classify before retrying — the four-way arbiter
- PLAN.md §Two pipelines — `graph` and `spec` produced together

## Contract

**Compile:** `graph` JSONB → `spec` JSONB → versioned `workflow_defs`. `graph` is what the canvas
reloads and `spec` is what the supervisor reads, **produced together so they cannot disagree**.

**Loop execution:** a `Loop`'s reset **starts a fresh run in the same session**, so memory, branch and
task linkage carry across because they belong to the session.

**Where `Verify` runs: a fresh container from the agent's own image, on the run's branch** — never in
the agent's container. Two reasons that are the same reason: the agent's container may already be gone,
and its filesystem holds whatever the agent did outside git. **A check that passes only on the machine
that made the change has verified nothing.** Exit code is the result; stdout and stderr are the evidence
the board requires for Done.

**`Condition` is a small total expression language** over facts the run already produced:
```
verify.passed · verify.exit_code · iteration · elapsed · tokens.used
events.count(tool_error) · events.last(kind) · artifact.exists(kind)
task.status · mail.pending
```
with `== != < > <= >=`, `and or not`, and parentheses. **Every operand is a column**, so a `Condition`
is a `WHERE` clause against the run — evaluable in the core in microseconds and reproducible from stored
events. It is deliberately not a scripting language, and **anything it cannot express is a `Gate`**.

**The four-way arbiter, before retrying.** Every guardrail answers a failed iteration the same way — try
again, then give up — and that is wrong for at least half of failures:

| Class | What Locus does |
| --- | --- |
| **Bug** | retry the iteration, and promote the failing check into the task's regression set |
| **Spec gap** | back to planning as an amendment — a *new* task for the delta, since the original may be Done |
| **Noise** | recalibrate the check; **do not count the iteration against `max_iterations`** |
| **Ambiguity** | refine the requirement, then restart — **never retry the implementation** |

**Noise not spending the iteration budget** is the consequence worth having on purpose: three flaky
failures otherwise kill a workflow at 8 iterations having attempted the work five times. And **spec gaps
and ambiguity leave the workflow entirely**, which is the only path that reaches the thing actually
broken.

The arbiter's classification is **a column on the iteration**, so spec-gap rate and ambiguity-detection
rate are queries — and a workflow that keeps producing spec gaps is visibly a planning problem rather
than a builder problem.

**The Ralph loop is a preset, not the only shape.** `locus ralph --goal … --verify …` runs one without
opening the canvas. Two honest notes: it is **token-hungry by construction**, and it is **only as good
as its verify** — a loop iterating against a weak check converges confidently on the wrong thing.

**The lead is deterministic.** Decomposition is what you drew, not what a model decided this run.

## Acceptance

1. `graph` and `spec` are produced together; a `spec` that disagrees with its `graph` is impossible by
   construction, not by convention.
2. A loop reset starts a **new run in the same session**, and memory, branch and task carry across.
3. `Verify` runs in a **fresh container on the run's branch** — a test proves it fails when the change
   exists only in the agent's container.
4. Every `Condition` operand resolves to a stored column; an expression with an unknown operand is
   refused at compile time.
5. A `Condition` is reproducible — re-evaluating against stored events gives the same answer.
6. The arbiter classifies a failure into one of four classes, recorded on the iteration.
7. A noise-classified failure does **not** decrement the iteration budget.
8. A spec gap produces a new task for the delta and leaves the workflow.
9. An ambiguity restarts after requirement refinement rather than retrying the implementation.
10. `locus ralph` runs a loop with no canvas.
11. No model is invoked anywhere in the orchestration path.

## Open

- What the arbiter itself costs. It is an agent with a bounded job, so every failed iteration now pays
  for a classification — worth it if it saves a retry, and PLAN.md does not say what the budget is.
