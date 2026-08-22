# screens-workshop

> **Historical M0.5 contract.** V2 expands Workshop and separates workflow Governance; new work
> follows `.specs/design-v2/spec.md`.

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · Views `extensions`, `agents`, `canvas`, `harnesses`

## Purpose

Where the meta-harness lives. PLAN.md says this deserves a place rather than a settings page, because
"authored once here, materialized fresh into every runtime" is the product's central claim — and the
screen that makes the claim inspectable is the one that proves it.

Tabs: `[Extensions, Workflow, Harnesses]`. **Agent definitions is a drill-down of Extensions, not a
tab** — see `navigation`.

## Governed by

- PLAN.md §The one surface — the eight extension types; linters and output-styles as the two exceptions
- PLAN.md §Materializers — the strategies and `weaker_than_native`
- PLAN.md §Model routing — mechanism in the file, policy in the UI
- PLAN.md §The Workflow Canvas — node vocabulary, guardrails, `Condition` as a total language
- `.specs/design-v2/spec.md` §Provider, harness, and tool policy

## Contract

### Extensions

Header: "The one surface" + "Eight extension types, authored once here, materialized fresh into every
runtime at run start", a search field, and a primary "New".

**4x2 grid of type cards** at radius 9: icon, lowercase type name, a 20px count, an 11px description,
and a footer line of native vs downgraded counts — `--bad` when downgrades dominate. Types: agents,
skills, rules, base-context, commands, hooks, output-styles, linters.

**The `agents` card is the entry card** — accent ring, pointer cursor, accent `arrow-right` — and
navigates to agent definitions.

Below: `RECENTLY EDITED` (type `.tag-neutral` chip at min-width 82px, file, change summary, age) and a
`MATERIALIZATION` card with an amber hairline: the byte-determinism note plus three figures — entries,
downgrades, cache read.

**Those figures are computed from `harnesses/*.toml`, never typed.** The handoff's own copy says
88/27; the files say 88/29 (eleven harnesses). Anything hardcoded here is wrong the next time a harness
is registered.

### Agent definitions (drill-down)

**196px sidebar**: `← Extensions` back link (accent, 10.5px, `arrow-left`), then `AGENT DEFINITIONS` —
builder (selected, `--sf2` + `--line2` ring), reviewer, interviewer, researcher, auditor, keeper, each
with a mono version. Footer: "**Markdown plus a tool list. No canvas, no compile.**"

**Main**: header `builder.md` + mono "v4 · edited 2h ago · used by 5 sessions", with "Diff v3" and
"Save as v5". Body: a frontmatter block on `--sf` with `border-left:2px solid var(--ac)`, mono
12px/1.72, keys in `#8fb8d6` — harness, model_tier, tools, skills, rules, memory_scope — followed by
Inter prose at 13px/1.65, max 660px. Footer: materialized to `/locus/config/agents/` for N harnesses,
M downgraded.

**"Save as v5" rather than "Save".** An agent definition is immutable once a run references it; edits
create a version. The button says so.

### Workflow

Three panes: **180px palette · 890px canvas · flexible inspector**.

- **Palette** (`--bg-deep`): draggable node chips (`cursor:grab`, `dots-six-vertical`) — Goal (amber
  hairline), Agent, Task, Loop, Condition (`#8fb8d6`), Gate, Verify (marked `req`). Then `PRESETS`
  (Ralph loop, Review pass) on `--sf2` and "A preset expands into ordinary nodes, so it can be edited
  rather than configured."
- **Canvas**: 24px dot grid, an SVG edge layer with four arrow markers, a dashed loop-back edge, a
  dashed rounded rect grouping the loop, and absolutely positioned node cards (radius 9, tinted 5x9px
  header strip with icon, 9.5px uppercase kind, right-side state). Edge labels are small pills on
  `--bg` with tinted hairlines. Bottom-left: a zoom pill and **"No model in the orchestration path —
  the graph decides"**.
- **Inspector**: node header; an expression builder of 26px mono token fields with an accent `and`
  joiner and a ghost "add clause"; a compiled-expression card with an `--ok` hairline ("total ·
  evaluable in the core · reproducible from stored events"); `OPERANDS — EVERY ONE IS A COLUMN` chips
  with "No code, no model, no I/O — anything this cannot express is a Gate."; then `GUARDRAILS` —
  max_iterations stepper (8), reflection-before-retry toggle, kill & reassign stepper (3), idle
  detection 60s, wall-clock none, token budget none, plus the budget note and "Waiting ≠ idle". Footer:
  "Validate graph" / "Save workflow".

At M0.5 this is the layout and the inspector on fixture data. Real graph editing, compile, and the live
overlay are `workflow-canvas` at M4, gated by Spike 3.

### Harnesses

Header: "Registered harnesses **12**" + "Mechanism lives in the file; policy lives here. Every harness
has every capability — only the mechanism differs." A legend (accent native, `rgba(212,97,79,.55)`
downgraded, "each names its loss") and "Register a harness".

**4-col card grid**, one per harness. Head: name 13px/500, mono id, and a mechanism badge (`hooks`
accent tint; `hooks · TS ext` / `hooks · py plugin` on `#314454`; `ACP` on `rgba(143,184,214,.18)`
`#8fb8d6`). Body: the injection line; a 4-row model-tier grid (low/med/high/xhigh, mono values, high in
accent, `↑ high` fallbacks in `--mu2`); and an **8-segment capability bar**, one per extension type —
accent native, red downgraded — over `8 extensions` and the downgrade count (`--bad` at 4+). Heavy
downgrades give the card a red hairline.

Footer: the computed downgrade line and "`tui = false` is required on all 12; a harness claiming true
is refused at registration."

**The `↑ high` fallback marker is not decoration.** A missing tier falls back **up, never down** —
falling down would answer a hard question with a cheap model and look like a bad agent rather than a
bad setting.

## Acceptance

1. Extensions shows exactly eight type cards.
2. Every count on Extensions and Harnesses is computed from `harnesses/*.toml` — no literal appears in
   the source. Adding a thirteenth TOML changes both screens with no edit.
3. Harnesses reports 11 harnesses and 29 downgrades.
4. The `agents` card navigates to the drill-down; the Extensions tab stays lit and the back link works.
5. The agent editor's save action reads "Save as v5" — a new version, never an overwrite.
6. Each harness card's capability bar has exactly 8 segments and its per-type colors match that
   harness's TOML.
7. A tier with no mapping shows the `↑ high` up-fallback marker, never a down-fallback.
8. The workflow inspector's operand chips are all drawn from the `Condition` operand list, and the
   panel states that anything unexpressible is a Gate.

## Open

- PLAN.md §Navigation lists **settings and the marketplace** as Workshop contents, but the handoff's
  Workshop tab bar has only three tabs and neither appears. Either they are drill-downs like agent
  definitions, or the tab set grows. Undecided, and it does not block M0.5.
