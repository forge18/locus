# spike-workflow-canvas

**Milestone** M0 · **Depends on** none · **Blocks** `workflow-canvas`, `wiki`

## Purpose

`solid-flow` carries the whole authoring surface for workflows and rates **Medium, not High** on
Context7. PLAN.md flags this as a risk with a named fallback — render the canvas directly with Solid
over `@dagrejs/dagre`. This spike decides between them before M4 commits, and it matters twice over
because the wiki's graph view reuses the same renderer.

## Governed by

- PLAN.md §The Workflow Canvas — node vocabulary, the loop, `Condition` as a total expression language
- PLAN.md §Two pipelines — `graph` JSONB as authored, `spec` JSONB as executed, produced together
- PLAN.md §M0, Spike 3
- PLAN.md §Risks — "Risk — `solid-flow` maturity"

## Contract

Delivers `spikes/03-workflow-canvas/FINDINGS.md` answering:

1. **Custom nodes.** Do typed node props and typed handles work, well enough to build the eight node
   kinds (`Goal`, `Agent`, `Task`, `Loop`, `Condition`, `Gate`, `Verify`, plus guardrails on the
   workflow) at the fidelity the handoff's screen 12 draws?
2. **Round-trip.** Does a graph survive `canvas → JSONB → canvas` **unchanged** — node positions,
   edges, handles, and the dashed loop grouping included? A canvas that reopens differently than it was
   drawn is one nobody will trust.
3. **Loop termination at compile time.** Can a loop with no termination condition be refused when the
   graph is saved rather than when it runs? PLAN.md makes this a graph-validation requirement.
4. **Verdict.** Does `solid-flow` carry this, or is the dagre fallback the cheaper path? State the cost
   of the fallback honestly — PLAN.md calls it "more work, no dependency risk".

## Acceptance

1. `spikes/03-workflow-canvas/FINDINGS.md` exists with a **binary recommendation**, not a comparison.
2. A canvas renders at least four distinct custom node types with typed props and named handles.
3. A graph serialized to JSONB and reloaded is byte-identical to the original — the test asserts on the
   serialized form, not on how it looks.
4. A loop with no termination condition is rejected by a validation function, with a message naming the
   offending node.
5. If the verdict is the fallback, the finding includes a working dagre-laid-out proof, not only an
   assertion that it would work.

## Open

- Whether the wiki graph view can share this renderer or needs its own. PLAN.md assumes sharing makes
  it "a palette, not a subsystem"; the spike should say whether that holds.
