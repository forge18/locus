# spike-workflow-canvas — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Write `spikes/03-workflow-canvas/QUESTION.md` — the four questions and the fallback's stated cost | — | `test -s spikes/03-workflow-canvas/QUESTION.md` |
| 2 | Solid + `solid-flow` scaffold in the spike dir | 1 | `pnpm -C spikes/03-workflow-canvas build` |
| 3 | Four custom node types with typed props and named handles: `Goal`, `Agent`, `Condition`, `Verify` | 2 | `pnpm -C spikes/03-workflow-canvas test -- custom-nodes` |
| 4 | Style them to the handoff's screen 12: tinted 5x9px header strip, kind label, right-side state | 3 | `test -s spikes/03-workflow-canvas/screenshots/canvas.png` |
| 5 | Edge layer with the four arrow markers and a dashed loop-back edge | 3 | `pnpm -C spikes/03-workflow-canvas test -- edges` |
| 6 | Dashed rounded-rect grouping around the loop, positioned from its member nodes | 5 | `pnpm -C spikes/03-workflow-canvas test -- loop-group` |
| 7 | Serialize the graph to JSONB and reload it | 3,5 | `pnpm -C spikes/03-workflow-canvas test -- roundtrip-loads` |
| 8 | Assert the round-trip is byte-identical on the serialized form | 7 | `pnpm -C spikes/03-workflow-canvas test -- roundtrip-identical` |
| 9 | `validateGraph()` refusing a loop with no termination condition, naming the node | 6 | `pnpm -C spikes/03-workflow-canvas test -- reject-nonterminating` |
| 10 | Same function refusing a cycle and a missing `verify` | 9 | `pnpm -C spikes/03-workflow-canvas test -- reject-cycle-and-missing-verify` |
| 11 | Repoint the renderer at a wikilink graph to test the shared-renderer claim | 3 | `pnpm -C spikes/03-workflow-canvas test -- wiki-graph` |
| 12 | If any of 3-10 fail: build the dagre fallback proof with the same four node types | 3,10 | `pnpm -C spikes/03-workflow-canvas test -- dagre-fallback` |
| 13 | Write `FINDINGS.md` with a binary recommendation and the shared-renderer verdict | 4,8,10,11 | `bash spikes/03-workflow-canvas/check-findings.sh` |
