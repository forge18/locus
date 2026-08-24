# planning-module

> Superseded by `plan-revision` for the stage list only; the agent roles, synthesis, ratchet, and traceability remain authoritative.

**Milestone** M5 · **Depends on** `acp-client`, `board`, `wiki`, `screens-plan`

## Purpose

A guided conversation that produces a reviewable plan. **Nothing it produces reaches the board until you
approve it, once, at the end.**

**Three agents, not one**, and the split is structural rather than stylistic: single-pass agents
terminate early by their own choice, because exploration and implementation compete for the same
attention and implementation wins. Splitting them lifted test pass rates 6.9-21.3% relative. Separately,
**capability decoupling** — strong code generation does not translate to effective clarification, and
more reasoning effort yields only marginal gains at spotting ambiguity.

## Governed by

- PLAN.md §The planning module — the whole section
- PLAN.md §The board — where approved tasks land
- PLAN.md §Memory — `task_class` and retrieval depth

## Contract

| Agent | `task_class` | Job |
| --- | --- | --- |
| **Interviewer** | `plan` | Drives the questions, holds state, writes the artifacts |
| **Researcher** | `research` | Facts, prior art, feasibility. **Never asked for intent** |
| **Auditor** | `plan` | Fresh context, adversarial. Grades the spec it did not write |

**The eight-step sequence:** Inputs → Orient → Converse → Synthesise → Audit → Recommend → Override →
Approve.

**The goal is an input, not an output.** That is what makes the question loop work: topics are ranked by
how much they bear on the goal, dropped when they do not, and the interview ends when nothing unresolved
still does. Without a stated goal every topic is equally relevant and the ranking is arbitrary.

**Synthesis is two passes, and the second one subtracts.** A completeness pass alone over-specifies, and
the failure that follows is specific: **downstream agents treat an unsupported requirement as
mandatory**, so a speculative clause becomes work someone does. The reduction pass exists to delete it
before it is load-bearing.

**Scope changes are approvals, never automatic.** Research can widen scope or narrow it, and **both
require you**. Narrowing is the more dangerous direction: an increase is visible in a growing spec,
while a silent reduction ships something missing and leaves no trace of which step removed it. They are
counted separately from questions, and a rejected increase is recorded as a `decision` so it is not
re-proposed.

**The audit loops back at most once.** A finding that says "this is ambiguous" is really saying "a
question was missed". But clarification quality degrades as ambiguity density rises, so a third pass
rarely helps where the second did not. Whatever survives becomes a named weakness, not a blocker.

**One criterion is mechanised rather than judged.** *Unambiguous* asks whether two readers would reach
the same understanding — so give the requirement to two agents with no shared context, ask each to
restate it, and **diff the restatements. Divergence is the ambiguity, and it names itself.**

**Effort scales by a ratchet, not a question.** Blast radius cannot be assessed up front, because
determining it is what planning does. So planning **starts minimal and escalates on evidence, never
de-escalating**: more than one repo, a scope decision raised, no prior art found, an answer
contradicting an earlier one, or unresolved topics outnumbering resolved ones.

**Re-planning: amend, never supersede.**

| Task state | Rule |
| --- | --- |
| Not started | rewrite in place |
| **In progress** | flag it and notify the session — never silently mutate a task an agent is working from |
| **Done** | never touch. Emit a *new* task for the delta, linked to the original |
| Requirement deleted | close as `superseded`, never delete — the trace has to survive |

**Traceability, both directions:** `excerpt → requirement → task → run → evidence → PR`. Excerpts follow
the W3C Web Annotation model — a quote selector beside a position selector, because quote-anchoring
survives re-rendering and offsets are precise. **An `exact` matching more than once is flagged for audit
rather than silently anchored to the first hit.**

**Outputs:** spec (a wiki page), tasks (vertical slices, **hardest first**, dependency edges drawn,
verify each), tools, a **proposed** workflow, and the recommendation you approve.

The workflow is proposed rather than committed because **a graph you did not lay out is one you will not
trust**.

## Acceptance

1. The three agents run in separate containers with no shared context.
2. The researcher is never asked for intent — asserted on its prompts.
3. The auditor sees a fresh context and did not write what it grades.
4. Synthesis runs two passes and the second **removes** clauses — a test asserts the spec shrinks.
5. A scope change stops for human approval in both directions, and is counted separately from questions.
6. A rejected scope increase is recorded as a `decision` and not re-proposed on the next pass.
7. The audit loops back **at most once**; a second finding becomes a named weakness.
8. The two-reader test runs on each requirement, and divergence is reported as the ambiguity.
9. The ratchet escalates on each of the five triggers and never de-escalates.
10. Approval lands tasks on the board; rejection keeps the draft.
11. Re-planning obeys all four task-state rules — the Done case emits a delta task rather than mutating.
12. A duplicate `exact` excerpt match is flagged, not silently anchored.
13. Tasks are ordered hardest first.

## Open

- PLAN.md records the **"I don't know what I want yet" entry point as undecided**. A goal is required up
  front, so something has to turn a vague idea into one, and whether that is a mode of this module or a
  separate one is not settled.
