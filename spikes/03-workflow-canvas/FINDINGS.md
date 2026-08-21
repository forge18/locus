# Spike 3 — workflow canvas · FINDINGS

Questions and decision rules were fixed in [QUESTION.md](QUESTION.md) before any of this ran.

**VERDICT: solid-flow.** It carries the canvas. Two defects were found and both have three-line fixes;
neither is a reason to pay for the dagre fallback, and one of them would have shipped silently.

| | Question | Verdict |
| --- | --- | --- |
| Q1 | Custom nodes — typed props, typed handles, screen-12 fidelity | **Yes** |
| Q2 | Round-trip `canvas → JSONB → canvas` unchanged | **Yes** |
| Q3 | Loop with no termination refused at save time | **Yes** |
| Q4 | solid-flow, or the dagre fallback | **solid-flow** |
| Open | Can the wiki graph view share this renderer | **Yes** |

Reproduce: `pnpm install && pnpm build && pnpm test && node scripts/screenshots.mjs && node
scripts/webkit-check.mjs`. 24 tests, `screenshots/canvas.png`, `screenshots/webkit.png`.

**The package is `@dschz/solid-flow`, not `solid-flow`.** PLAN.md's `/dsnchz/solid-flow` is the right
GitHub repo and the wrong npm name — `solid-flow` on npm is an unrelated port by a different author.
Worth correcting in PLAN.md, because installing the wrong one would look like it worked.

---

## Q1 — Custom nodes

**VERDICT: yes.** Four node kinds render from their own typed data with named handles, at the density
screen 12 draws. `test/custom-nodes.test.tsx`, 6 tests; `screenshots/canvas.png`.

`NodeProps<T>` gives typed props, and each kind reads its handle ids from the same `HANDLES` table the
validator uses — so a handle cannot be drawn that `validateGraph` would then call unresolved. The
Condition renders two distinguishable outbound handles, `true` and `false`, which is the part the whole
node vocabulary rests on: a Condition routes deterministically, so its edges are not interchangeable.

Fidelity reached without fighting the library: 9px tinted header strip, uppercase kind label, right-side
state, mono expression, chips, the 24px dot grid, and four arrow markers. The dashed loop grouping is
derived from its member nodes rather than stored, so it cannot drift out of sync with which nodes are
actually in the loop.

## Q2 — Round-trip

**VERDICT: yes, byte-identical.** `test/roundtrip-identical.test.ts` asserts on the serialized form, not
on the rendering — a screenshot comparison would pass on a graph whose handles had silently collapsed.

`deserialize → serialize` is a fixed point, positions and handle identities survive, and the check has
teeth: changing one `sourceHandle` from `true` to `false` changes the bytes.

Nothing about this depends on solid-flow. `src/graph.ts` defines the graph; `src/Canvas.tsx` renders it.
That separation is deliberate and it is the reason Q4's downside is bounded — see below.

## Q3 — Loop termination at compile time

**VERDICT: yes.** `validateGraph()` refuses a loop that can only ever run again, at save time, naming the
node: `Loop 'build loop' (l-1) has no termination condition: its 'exit' handle is unwired, no node in
its body routes out of it, and it sets no max_iterations`.

Three termination routes are accepted — a wired `exit` handle, a node in the body routing out, or
`max_iterations`. The dashed loop-back edge is explicitly **not** counted as a way out; counting it would
make every non-terminating loop look terminated.

Cycles and missing `verify` are refused the same way, and a *declared* loop-back is not an error — a
workflow is a loop toward a goal, so the thing worth refusing is an undeclared back edge.

This spike implements three rules. PLAN.md lists more (unresolved handles, unreachable goal, role
contamination) and they belong to `.specs/workflow-canvas`; what is settled here is that graph
validation is a property of the drawing and needs nothing from the renderer.

## Q4 — The verdict

**VERDICT: solid-flow.** The dagre fallback is not built, because task 12 fires only if any of tasks
3–10 fail and none did. Building a fallback nobody needs would be the exact cost PLAN.md was trying to
avoid.

### Two defects found, both fixed here

**1. `requestIdleCallback` on WebKit — this one would have shipped silently.**

solid-flow calls `requestIdleCallback` unguarded in `requestUpdateNodeInternals`. WebKit does not
implement it. Measured in playwright's WebKit 26.5 against the built canvas:

```
webkit    requestIdleCallback: undefined   nodes: 0  edgePaths: 0  loopGroups: 0
          pageErrors: ["ReferenceError: Can't find variable: requestIdleCallback"]
chromium  requestIdleCallback: function    nodes: 5  edgePaths: 8  loopGroups: 1
```

**The canvas renders nothing.** Not degraded — nothing, on one ReferenceError with no other symptom.

WebKit is the engine behind **WKWebView (Tauri on macOS)** and **WebKitGTK (Tauri on Linux)**. So this
is two of Tauri's three platforms, and it is the concrete instance of the risk PLAN.md §Risks names as
"if CodeMirror or `solid-flow` misbehave there". The three-line polyfill in `src/main.tsx` fixes it
completely — after it, WebKit and Chromium render identically (5 nodes, 8 edge paths, 1 loop group, no
errors). `screenshots/webkit.png` is the after.

**This is Spike 2's problem too.** Any dependency may reach for a platform API WebKit lacks, and a
Chromium-only check will not find it. `.specs/spike-editor-embed` task 10 asks for a WebKitGTK verdict;
this is evidence that the question is not theoretical, and that `scripts/webkit-check.mjs` is the shape
of the check — run the real build in a real WebKit and read `pageErrors`.

**2. `ViewportPortal` is broken in 0.1.4 and fails silently.**

It is the documented way to place content in graph space so it shares the pan/zoom transform — which is
what the dashed loop grouping needs. It mounts into `.solid-flow__viewport-portal`, an element this
version never renders. The selector returns `null`, `mount` becomes `undefined`, and Solid's `Portal`
falls back to `document.body`. Reproduced: the group's parent chain came back `DIV → BODY → HTML`, and
the rectangle sat in screen space, detached from the nodes it was supposed to enclose.

No error is raised. The workaround is to read `useViewport()` and apply the transform directly, which is
what `src/Canvas.tsx` does now and what the screenshot shows working.

### Why these do not change the verdict

Both are shallow, and the alternative is not cheap. `@dagrejs/dagre` lays out; it does not render and it
does not do interaction. Node dragging, edge routing, pan/zoom, selection, handle hit-testing, snap,
multi-select, edge re-targeting, and undo across all of it would be written by hand — against a library
that gets all of it from `@xyflow/system`, the same core React Flow and Svelte Flow are built on.

The exposure is real and should be recorded rather than argued away:

- **0.1.4, last published 2025-08-30 — twelve months without a release**, five versions ever, ~1,300
  downloads a month. Both defects above are consistent with that.
- Its hard parts are not its own. `@xyflow/system` does the transforms, edge maths, and handle logic;
  what is thin is the Solid layer on top. That is the better half to be thin.

**Watch it, do not pay it down.** If a third defect of this class appears at M4, the fallback argument
gets stronger — and because `src/graph.ts` owns serialization and validation with no renderer
dependency, switching costs the interaction layer and nothing else. That separation is what makes the
verdict reversible, and it should survive into `.specs/workflow-canvas`.

## Open — can the wiki graph view share this renderer?

**VERDICT: yes. PLAN.md's "a palette, not a subsystem" holds.**

`test/wiki-graph.test.tsx` points the same `<SolidFlow>` at a wikilink graph — pages as nodes,
`[[wikilinks]]` as edges — changing only the node component. Nothing else differed: same store
constructors, same handle model, same fitView.

One behaviour worth carrying into `.specs/wiki`: a wikilink to a page nobody has written is **not** an
edge. It is the wiki's own affordance for "worth writing later", and rendering it as a dangling edge
would turn an intentional signal into visual noise.

---

## What this spike does not answer

- Interaction fidelity. Nothing here drags a node, re-targets an edge, or undoes anything — jsdom cannot
  lay out, so edge geometry is verified by screenshot rather than by assertion. Multi-select and undo are
  `.specs/workflow-canvas`'s risk and are not de-risked by this.
- The `spec` JSONB. This round-trips `graph`; the executable half is `workflow-engine`'s.
- Whether the wiki graph view ships at M5 — only that it *can* share this renderer.
- WebKitGTK specifically. Playwright's WebKit is a WebKit build with its own embedding layer, not GTK's.
  The engine-level answer transfers; a windowing-level one does not.

## What this spike changes elsewhere

| Where | Change |
| --- | --- |
| PLAN.md §The Workflow Canvas | The npm package is `@dschz/solid-flow`; `/dsnchz/solid-flow` is the repo, not the package |
| `.specs/workflow-canvas` | Ship the `requestIdleCallback` polyfill before any solid-flow import, and keep it out of a lazy chunk |
| `.specs/workflow-canvas` | Do not use `ViewportPortal` on 0.1.4; apply `useViewport()` directly for graph-space overlays |
| `.specs/workflow-canvas` | Keep serialization and validation renderer-independent — it is what keeps the fallback affordable |
| `.specs/spike-editor-embed` | Run the real build in a real WebKit and read `pageErrors`; `scripts/webkit-check.mjs` is the pattern |
| `.specs/wiki` | The graph view shares this renderer. An unresolved wikilink is not an edge |
| `.specs/ci` | A Chromium-only browser check will pass while two of three Tauri platforms are broken |
