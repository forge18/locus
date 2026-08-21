# Spike 3 — workflow canvas · QUESTION

Written before the experiment. Nothing below is a conclusion.

**Governed by** PLAN.md §The Workflow Canvas, §Two pipelines, §M0 Spike 3, §Risks — "Risk —
`solid-flow` maturity". Contract in `.specs/spike-workflow-canvas/spec.md`.

## The unknown

`solid-flow` (`/dsnchz/solid-flow`) carries the **entire** workflow authoring surface. PLAN.md chose it
over Rete.js on one fact: Rete's renderers target React, Vue, and Svelte, but not Solid. That is a
sound reason to pick it and no reason at all to trust it — PLAN.md rates it **Medium, not High** on
Context7 and names the fallback in the same breath: render the canvas directly with Solid over
`@dagrejs/dagre` for layout, "more work, no dependency risk".

The exposure is doubled, not single. PLAN.md §The wiki calls the graph view "nearly free" precisely
*because* `solid-flow` is already in the bundle — pages are nodes, `[[wikilinks]]` are edges. If the
renderer does not carry the canvas, it does not carry the wiki graph either, and a second feature
loses its cheapest property.

## The four questions

### Q1 — Custom nodes

Do typed node props and typed handles work well enough to build the node vocabulary at the fidelity
the design handoff's screen 12 draws?

The vocabulary is eight: `Goal`, `Agent`, `Task`, `Loop`, `Condition`, `Gate`, `Verify`, plus
`Guardrails` attached to the workflow. Four are exercised here — `Goal`, `Agent`, `Condition`,
`Verify` — because they span the shapes that differ: a node with an approval state, a node carrying a
pinned version and a permission narrowing, a node with **multiple outbound handles that must be
distinguishable by name**, and a node whose state is pass/fail.

Typed handles are the part that matters. A `Condition` routes deterministically, so its edges are not
interchangeable: `true` and `false` must be separate named handles that survive serialization, or the
graph cannot say which branch it drew.

### Q2 — Round-trip

Does a graph survive `canvas → JSONB → canvas` **unchanged** — node positions, edges, handle
identities, and the dashed loop grouping included?

PLAN.md §Two pipelines makes `graph` JSONB "what the canvas reloads" and `spec` JSONB "what the
supervisor reads", produced together so they cannot disagree. A canvas that reopens differently than it
was drawn breaks the first half of that and quietly undermines the second.

**The assertion is on the serialized form, not on the rendering.** A screenshot comparison would pass
on a graph whose handles had silently collapsed.

### Q3 — Loop termination at compile time

Can a loop with no termination condition be refused **when the graph is saved**, rather than when it
runs?

PLAN.md makes this graph validation, alongside cycles, unresolved handles, missing `verify`,
unreachable goal, and role contamination. Every one of those is a property of the drawing, and the
whole argument for deterministic routing — "no model in the orchestration path", the graph decides —
depends on the graph being checkable before anything executes. A validation that only fires at run
time is a guardrail, and PLAN.md already has guardrails; this is meant to be the earlier gate.

The error must **name the offending node**. A validator that says "invalid graph" on a canvas of forty
nodes has not helped anyone.

### Q4 — Verdict

Does `solid-flow` carry this, or is the dagre fallback the cheaper path?

**A binary recommendation, not a comparison.** The acceptance criterion says so, and the reason is that
M4 cannot start on "it depends". If the answer is the fallback, the finding carries a **working
dagre-laid-out proof** of the same four node types — because "the fallback would work" is exactly the
kind of claim a spike exists to stop being made on paper.

## What decides it

Recorded now, so the verdict is a decision and not a preference:

| Result | Verdict |
| --- | --- |
| Q1, Q2, Q3 all hold | `solid-flow`. The dependency risk stays a risk and is watched, not paid down. |
| Q2 fails | Fallback. Round-tripping is not a feature that can be patched around — it is the storage contract. |
| Q1 fails on typed handles | Fallback. Named handles are what makes `Condition` routing expressible at all. |
| Q1 fails only on styling fidelity | `solid-flow`, with the gap recorded. A header strip is CSS, not a renderer property. |
| Q3 fails | Neither — Q3 is validation over the serialized graph and is the renderer's problem only if the renderer will not give up a serializable graph. If it fails, that is really a Q2 failure wearing a different hat. |

## The fallback's cost, stated before it is needed

- Node dragging, edge routing, pan/zoom, selection, and handle hit-testing are all written by hand.
  `@dagrejs/dagre` lays out; it does not render and it does not do interaction.
- Interaction fidelity is where the time goes and it is easy to underestimate: snap, multi-select,
  edge re-targeting, and undo across all of it.
- What is gained is real: no dependency risk on a Medium-rated library that carries two features, and
  a serialization format that is ours by construction rather than by adaptation.

## What this spike does not decide

- The `Condition` expression language itself. PLAN.md fixes its operands and operators; parsing it is
  `workflow-engine`'s work, not the canvas's.
- The `spec` JSONB shape. This spike round-trips `graph`; `spec` is produced by the same pipeline and
  is settled in `workflow-engine`.
- Whether the wiki graph view **ships** at M5. Q1–Q4 answer only whether it *can* share this renderer,
  which is PLAN.md's "a palette, not a subsystem" claim.
