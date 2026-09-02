# plan-revision

> **Workspace wrapper revision:** `planning-workspace` now governs durable workspace lifecycle, page-owned scope, and approval/materialization. This spec remains authoritative for the seven-stage child-spec profile.

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision` · **Blocks** M1 planning runtime changes and `manage-revision`

## Purpose

Rewrite the Plan surface contract from the mockup's current seven-stage pipeline —
`docs/UI mockups for PLAN.md/Locus v2.dc.html` — replacing the eight-step, three-gate sequence
`planning-module` and `crates/locus-core` still ship. Audit and Override stop being stages: the
auditor becomes an agent role that runs on a schedule, and user override is expressed by editing
during Recommend and Decompose, not by a dedicated gate. This spec covers the whole Plan screen —
the stage strip and stepper, the All plans rail, the persistent Outputs rail, and the per-stage
contract for Inputs, Orient, Converse, Synthesis, Recommend, Decompose, and Approved.

## Governed by

- `PLAN.md` §The planning module — the three-agent sequence, the ratchet, traceability
- `docs/UI_MOCKUP_REVIEW.md` — the Plan section, and Decision 1 (seven stages)
- `.specs/design-revision/spec.md` — Decision 1 (the same seven-stage decision, restated here as
  this feature's contract rather than a settled vocabulary note)
- `.specs/planning-module/spec.md` — the agent roles, the two-pass synthesis, the traceability
  model, and the ratchet triggers, none of which this spec changes

## Contract

### Stage strip and stepper (cross-cutting)

The header carries an **All plans** toggle, a centred **Back / "Step n of 7 · Stage" / Next**
stepper, an **Outputs** toggle, then the plan's title and origin. Below it, a seven-item stage
strip — Inputs, Orient, Converse, Synthesis, Recommend, Decompose, Approved — where every stage is
individually clickable: a direct jump, not a sequence gate. Back and Next move one stage at a time
and derive their label from the same stage list the strip renders; Back is disabled on Inputs, Next
on Approved. Stage sub-labels: *what to add* (Inputs), *repos indexed* (Orient), *n questions*
(Converse), *spec draft, pass 2* (Synthesis), *spec + confidence* (Recommend), *what becomes a
card* (Decompose), *cards on the board* (Approved).

### All plans rail (cross-cutting)

Three sections, in order: **In progress**; **Drafts — rejected, kept here** (a rejected draft is
kept, not discarded, and carries its confidence and open counts); **Approved · on the board**. A
plan card shows title, step line, and project. **New plan** opens directly at Inputs — the goal is
an input, not an output, so nothing about a new plan is decided before that stage asks for it.

### Outputs rail (persistent)

Present on every stage: `spec.md` with requirement count, last-edit time, and **Edit** (jumps to
Recommend); a `tasks` preview with **Edit & decompose** (jumps to Decompose); a **recommendation**
card carrying the confidence value, `open[n]`, the ratchet verdict (e.g. "one more elicitation pass
before approval"), and **Approve — n to the board**. The recommendation card reads the same
`EditableSpec`/`changed_requirements` state Recommend edits — it never holds a second, driftable
copy of confidence or `open[n]`.

### 1 · Inputs

"What should this plan add?" One free-text goal, the target project, attached repos, and **Start
planning**. Footer: "One goal per plan. If this turns out to be two things, the conversation will
say so and you can split it there." The goal is required; project and target repo are required by
the underlying `planning-module` contract even though this stage's job is only to collect them, not
to justify them.

### 2 · Orient

Indexes the attached repos — symbols, call graph, history — before any question is asked: "The
conversation is only as good as what the index found, so this runs before any question gets
asked." Converse is unreachable by direct jump until indexing completes.

### 3 · Converse

Embeds the agent panel with header, research, plan tray, task link, and workflow suppressed. This
is the interviewer / researcher / auditor conversation from `planning-module`: the interviewer
drives, the researcher answers fact questions, and the auditor's findings render inline as messages
(already true of `Message.tsx`'s `msg-auditor` styling) rather than in a separate stage. A scope
decision — widen or narrow — renders inline in the message flow, never as a modal or a gate.

### 4 · Synthesis

Two passes: pass 1 drafts requirements for completeness, pass 2 cuts unsupported clauses so nothing
downstream reads as mandatory by accident. What survives the second pass that the two-reader test
still cannot resolve becomes `open[n]`, carried into Recommend rather than blocking here.

### 5 · Recommend — the spec editor

`spec.md` with version, requirement count, last edit, an **unsaved** badge, and a confidence chip
(`confidence 0.62 · open[2]`). Buttons **Revert**, **History**, **Save & re-synthesise**.

- Requirements carry **stable ids** (`R-05`, `R-06`, …) in a fixed left column, one editable block
  each, RFC-2119 language. A task already on the board keeps pointing at the requirement it came
  from, even after a rewrite.
- A requirement that closes a gap shows "closes open[1] — …" and a **Mark resolved** control.
- Requirements already carried by a board card are marked, so a rewrite shows what it is about to
  contradict.
- Per-section "+ Add requirement to §n".
- Outline rail with the five canonical sections — **Scope, Trust boundaries, Conflict resolution,
  Error conditions, Out of scope** — foot shows `open[n]` and "n requirement with no card", both
  derived from the requirement data, not a static string.
- **Save & re-synthesise** re-runs synthesis over the changed requirements only —
  `EditableSpec::changed_requirements()` and `mark_reaudited()` are the mechanism this button
  drives.

This is where the confidence ratchet and the `open[n]` gap machinery from `planning-module` live
now that Audit is not a separate stage: the ratchet's escalation triggers still fire during
Synthesis and Converse, but the number and the verdict surface here.

### 6 · Decompose

Three parts.

- **What becomes a card** — *The Spec* (one card, coarsest, agent decomposes at run time); *Every
  task* (one card per task, finest, dependencies carried); *Spec + carve-outs* (the spec rides as
  one card, expected-long tasks get their own — recommended). Card counts come from
  `CardMode::card_count`, one formula, never duplicated arithmetic in the view.
- **Runs as** defaults bar — Workflow, Harness, Model, Effort. Model and Effort stay disabled and
  read `auto-route` until a Harness is chosen; pinning either stops routing for that field only. A
  warning shows while workflow or harness is unset: "Pick a workflow and a harness — every task
  inherits them."
- **Tasks from the spec** — columns `id · Task · Runs as · After · On the board as · (expand)`.
  Title is editable; "Runs as" summarizes overrides or reads "plan defaults"; "On the board as"
  toggles between **its own card** and **rides on the spec card**. Expanding a row exposes per-task
  Workflow / Harness / Model / Effort and **Reset to defaults**. Footer states the resulting card
  count, "Sizing happens once, when the card reaches the board," and **Approve — n to the board**.

### 7 · Approved

Terminal summary: "Approved — n cards on the board." Four stat cards — Questions asked/answered/
deferred, Requirements across five sections, final Confidence with carried `open[n]`, Cards. A
**What happened** log, one row per stage with description and duration. A **Cards created** table
(id, title, workflow · harness). Exactly two actions: **Start a new plan**, **Open the board**. Post-
approval spec edits go through the board, not through this page: "The spec stays as the artefact
the cards point at." A new plan starts empty except the repo index, "which is already warm."

### Audit and Override are not stages (cross-cutting)

Neither name appears as a stage label, a tab, or a route segment anywhere under
`apps/desktop/src/screens/plan`. The auditor survives as an agent role — `task_class: plan`, fresh
context, grades a spec it did not write — that runs on a schedule and whose findings render inline
during Converse and Synthesis. User override is expressed by editing requirements during Recommend
and by editing the task table during Decompose; there is no dedicated override screen or gate.

### Core: `PlanningStage` nine → seven

`crates/locus-core/src/services/planning.rs` drops the `Audit` and `Override` variants and renames
`Synthesise` to `Synthesis` and `Approve` to `Approved`, matching `docs/UI_MOCKUP_REVIEW.md` and
`.specs/design-revision/spec.md` Decision 1 exactly. `PlanningStage::ALL` becomes `[Inputs, Orient,
Converse, Synthesis, Recommend, Decompose, Approved]`; `Recommend.next()` returns `Some(Decompose)`
directly, since nothing sits between them once Override is gone. `EditableSpec`, `Requirement`,
`ApprovedPlan`, `CardMode`, `Decomposition`, and `BoardCard` are unaffected — they already model
Recommend and Decompose correctly and need no change for this migration.

## Supersedes

| Existing feature | Replacement |
| --- | --- |
| `planning-module` (stage list only — `Inputs → Orient → Converse → Synthesise → Audit → Recommend → Override → Decompose → Approve`) | this spec's seven-stage list; `planning-module`'s three agent roles, two-pass synthesis, traceability model (`excerpt → requirement → task → run → evidence → PR`), and the five ratchet triggers are not superseded and still govern |

`planning-module` carries a pointer line to this spec, scoped to the stage list, so it cannot be
mistaken for describing the current Plan screen.

## Acceptance

1. `PlanningStage::ALL` has exactly seven variants — `Inputs, Orient, Converse, Synthesis,
   Recommend, Decompose, Approved` — and no `Audit` or `Override` variant exists in
   `crates/locus-core`.
2. `PlanningStage::Recommend.next()` returns `Some(Decompose)` directly.
3. Every stage label, tab, and route under `apps/desktop/src/screens/plan` reads one of the seven
   names; `Audit` and `Override` appear only in prose describing the auditor role and user-edit
   behavior, never as a stage name.
4. The stage strip renders all seven stages as individually clickable jump targets, and the
   Back/Next stepper reads "Step n of 7 · <Stage>".
5. The All plans rail groups plans into exactly the three sections — In progress; Drafts —
   rejected, kept here; Approved · on the board — each carrying its count, and New plan opens at
   Inputs.
6. Recommend's confidence chip and `open[n]` counter, and the Outputs rail's recommendation card,
   read the same underlying data, so a save never leaves the two disagreeing.
7. Decompose's Model and Effort controls stay disabled and read `auto-route` until a Harness is
   chosen, and pinning either stops routing for that field only, never the whole task.
8. The Approved stage shows the four stat cards, the per-stage "What happened" log, and the "Cards
   created" table, and offers exactly two actions — Start a new plan, Open the board.
9. `.specs/planning-module/spec.md` carries a "Superseded by `plan-revision`" pointer line naming
   the stage-list scope of the supersession.
10. No file under `.specs/plan-revision` or `crates/locus-core/src/services/planning.rs` describes
    an eight-step or nine-stage sequence.

## Open

- The autorouting effort vocabulary Decompose's "Runs as" bar cycles (`low / medium / high /
  xhigh`) does not match the six-band vocabulary Workshop → Plugins → Harness lists (which includes
  `minimal`). `.specs/design-revision` already flags this; the fix lands in `workshop-revision`,
  not here — Decompose adopts whatever vocabulary that spec settles on.
- `.specs/setup-revision`, `.specs/screens-plan`, and `.specs/design-desktop` still cite the old
  eight-step sequence in prose. Repointing them is out of this feature's blast radius; each is
  either already superseded by a different M0.7 feature or carries its own maintenance debt.
