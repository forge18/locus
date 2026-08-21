# workflow-canvas — tasks

Spike 3 gates this. A verdict for the dagre fallback rewrites tasks 1-6.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `solid-flow` (or the fallback) wired into `workflow-canvas/` | — | `pnpm -C apps/desktop test -- canvas/mounts` |
| 2 | `Goal`, `Agent`, `Task`, `Loop`, `Condition`, `Gate`, `Verify` node components | 1 | `pnpm -C apps/desktop test -- canvas/node-types` |
| 3 | Typed props and typed handles per node kind | 2 | `pnpm -C apps/desktop test -- canvas/typed-handles` |
| 4 | Edge layer with the four arrow markers and edge label pills | 1 | `pnpm -C apps/desktop test -- canvas/edges` |
| 5 | Dashed loop-back edge and the loop grouping rect | 4 | `pnpm -C apps/desktop test -- canvas/loop-visuals` |
| 6 | Pan, zoom, and the zoom pill | 1 | `pnpm -C apps/desktop test -- canvas/viewport` |
| 7 | Serialize to `graph` JSONB including positions | 3 | `cargo test -p locus-core workflow::graph_serializes` |
| 8 | Round-trip exactly — reopening reproduces the authored graph | 7 | `cargo test -p locus-core workflow::graph_roundtrip_exact` |
| 9 | Validation: reject cycles | 7 | `cargo test -p locus-core workflow::rejects_cycle` |
| 10 | Validation: reject unresolved handles | 7 | `cargo test -p locus-core workflow::rejects_unresolved_handle` |
| 11 | Validation: reject a missing `verify` | 7 | `cargo test -p locus-core workflow::rejects_missing_verify` |
| 12 | Validation: reject an unreachable goal | 7 | `cargo test -p locus-core workflow::rejects_unreachable_goal` |
| 13 | Validation: reject a non-terminating loop, at save time | 7 | `cargo test -p locus-core workflow::rejects_nonterminating_loop` |
| 14 | Validation: reject role contamination | 7 | `cargo test -p locus-core workflow::rejects_role_contamination` |
| 15 | Every rejection names the offending node | 9,10,11,12,13,14 | `cargo test -p locus-core workflow::rejections_name_the_node` |
| 16 | `Agent` node permission narrowing: a tools subset, network tier, write scope | 3 | `cargo test -p locus-core workflow::node_narrows` |
| 17 | Assert a node cannot grant a capability the definition lacks | 16 | `cargo test -p locus-core workflow::node_never_widens` |
| 18 | Palette with draggable node chips | 2 | `pnpm -C apps/desktop test -- canvas/palette` |
| 19 | Ralph preset expanding into ordinary editable nodes | 18 | `pnpm -C apps/desktop test -- canvas/preset-expands` |
| 20 | Inspector: expression builder and compiled-expression card | 2 | `pnpm -C apps/desktop test -- canvas/inspector` |
| 21 | Guardrails panel bound to the workflow's config | 20 | `pnpm -C apps/desktop test -- canvas/guardrails-bound` |
| 22 | Live overlay painting per-node state from normalized events | 8 | `pnpm -C apps/desktop test -- canvas/live-overlay` |
| 23 | Assert overlay and transcript read the same event source | 22 | `pnpm -C apps/desktop test -- canvas/one-event-source` |
| 24 | Generate board `blocked_by` edges from the graph | 8 | `cargo test -p locus-core workflow::edges_become_dependencies` |
