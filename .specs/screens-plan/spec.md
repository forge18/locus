# screens-plan

> **Historical M0.5 contract.** V2 adds editable Spec and Tasks & cards views; new work follows
> `.specs/design-v2/spec.md`.

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · View `plan`

## Purpose

The planning module's surface: a guided conversation that produces a reviewable plan. PLAN.md's rule
is that **nothing reaches the board until one approval at the end**, and this screen is where that
approval is given — so the recommendation and its confidence have to be legible enough to approve
honestly.

## Governed by

- PLAN.md §The planning module — three agents, the eight-step sequence, scope changes as approvals
- PLAN.md §Outputs — the recommendation object
- `.specs/design-v2/spec.md` §Project, plan, and dispatch policy

## Contract

Three panes: **216px list · flexible conversation · 296px outputs**.

**List.** Block primary "New plan" and the note that a plan starts from a goal, a target repo, and the
repos involved. Sections: `IN PROGRESS` (selected card with accent ring, `circle-notch`, "step 5 ·
audit", project right-aligned); `DRAFTS — REJECTED, KEPT HERE` with confidence and open counts;
`APPROVED · ON THE BOARD`, dimmed, with `--ok` "6 tasks landed". Footer: "Nothing reaches the board
until one approval at the end."

**Conversation.** An **eight-step breadcrumb** — Inputs → Orient → Converse → Synthesise → Audit →
Recommend → Override → Approve. Done steps get an `--ok` check, the current step is an accent pill,
future steps are `--mu2`.

Messages: 22px rounded-5 mono-initial avatar (`--blue` for agents, `#5c4413` for the auditor), a role
caption, and a bubble on `--sf` at max 600px. Your replies are right-aligned on `--sf3` at max 560.

- **Scope decision card** — inline, accent ring, `arrows-split` icon: "resolves inline, not as a
  separate gate", with "Widen scope" / "Keep out, note as open". This is the interaction PLAN.md
  insists is *not* a question and is counted separately.
- **Auditor findings** carry a red-tinted border.
- Live line: pulsing dot + "interviewer is re-opening question 14 of 14".
- Footer: input with a blinking accent caret and mono `ACP · session/prompt`.

**Right rail — `DRAFT OUTPUTS`.** `spec.md`, four numbered tasks, a tool list as mono `.tag` chips with
`+ pgvector` as `.tag-outline`, and the **recommendation card** (accent ring): 21px `0.62` confidence,
`open[2]`, the ratchet note, and block primary "Approve — 4 tasks to the board".

**Confidence is a named condition, not a number alone.** PLAN.md is explicit that *"medium; high once
the migration path is confirmed"* is an action and a percentage is not — so the card shows the figure
and the condition that would move it.

## Acceptance

1. The breadcrumb shows all eight steps with three distinct states, and the current step is derivable
   from fixture data rather than hardcoded.
2. Agent, auditor and human messages are visually distinguishable without reading them.
3. The scope-decision card renders inline in the conversation flow, **not** as a modal or a gate.
4. The recommendation shows both a confidence figure and the condition that would raise it.
5. The approve button states the task count it would land.
6. The list footer carries the one-approval rule verbatim.
7. Rejected drafts remain visible and reachable — they are kept here, not discarded.

## Open

- PLAN.md records the "I don't know what I want yet" entry point as **undecided** — a goal is required
  up front and something must turn a vague idea into one. The screen has no affordance for it, which is
  correct until that decision is made.
