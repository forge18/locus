# workflow-canvas

**Milestone** M4 · **Depends on** `spike-workflow-canvas`, `screens-workshop`, `workflow-engine`

## Purpose

Where orchestration becomes authorable, and where PLAN.md says the product's character arrives. Agents
already work — they are Markdown and shipped at M1. This is the graph.

**The workflow is also the team.** There is no `Team` entity: its `Agent` nodes are the roster, each
node carries a role, and the edges are the dependencies.

## Governed by

- PLAN.md §The Workflow Canvas — node vocabulary and graph validation
- PLAN.md §Teams — the workflow *is* the team
- PLAN.md §Two pipelines — `graph` as authored, `spec` as executed
- `.specs/design-v2/spec.md` §Workflow authoring

## Contract

`solid-flow`, with the dagre fallback if Spike 3 said so.

**Node vocabulary:**

| Node | Contributes |
| --- | --- |
| `Goal` | what the loop is *for*. **This is the approval gate** — a person approves it before the loop runs — and it is also the termination condition |
| `Agent` | an agent definition pinned by version, **plus a `role`**. Carries this role's permission narrowing: a `tools` subset, a `network` tier, `write` scope |
| `Task` | a unit of work, optionally sourced from the board |
| `Loop` | what repeats, and what resets between passes |
| `Condition` | deterministic routing. **No model in the orchestration path** |
| `Gate` | human, or another agent as reviewer |
| `Verify` | the runnable success criterion. **Required** |

**Graph validation refuses**, at save time and not at run time: cycles, unresolved handles, missing
`verify`, an unreachable goal, a loop with no termination, and **role contamination**.

**Role contamination is refused at compile time**, and belongs in validation rather than in a convention
someone follows. One agent definition may not hold both the builder and the tester role in one
workflow, and the reviewer may not be the implementer. **A verifier that wrote the code inherits its
assumptions, and a graph that quietly allows it produces reviews that agree with everything.**

**Permissions narrow, never widen.** An `Agent` node may take the reviewer's write access away; it
cannot grant a capability the definition did not have. If a graph could widen, every workflow would be
a place to re-grant privileges and reading an agent would stop telling you what it can do.

**Board dependency edges are generated from the graph.** Dependencies are declared once, visually,
rather than drawn here and re-entered on the board.

**A preset expands into ordinary nodes** so it can be edited rather than configured. The Ralph loop
ships as one.

**Live overlay:** opening a running workflow paints run state on the graph — which node is executing,
which `Verify` passed, which iteration, tokens and wall-clock per node. The terminal shows the agent
working; the canvas is the map. **Both read the same normalized events** — one source, two renderings.

## Acceptance

1. All seven node types render with typed props and typed handles.
2. A graph round-trips through JSONB **exactly** — reopening reproduces what was authored, positions
   included.
3. Validation refuses each of the six invalid graphs, naming the offending node.
4. A loop with no termination is refused **at compile time, not at run time**.
5. A graph handing one agent both builder and tester roles is refused.
6. An `Agent` node cannot grant a tool the definition lacks; it can remove one.
7. Dropping a Ralph preset expands it into ordinary editable nodes.
8. The live overlay paints per-node state from the same events the transcript reads.
9. Tasks created by a workflow inherit `blocked_by` from the graph's edges.

## Open

- A workflow cannot re-plan itself mid-run, and PLAN.md states that cost plainly. If dynamic
  decomposition turns out to be needed, the fix is an agent that *authors a workflow* and submits it for
  goal approval — not a model in the execution path.
