import { SolidFlow, Background, Controls, MarkerType, useViewport,
         createNodeStore, createEdgeStore } from '@dschz/solid-flow';
import { For, createMemo } from 'solid-js';
import { nodeTypes } from './nodes/WorkflowNodes';
import { loopGroups } from './loopGroup';
import type { WorkflowGraph } from './graph';

// The handoff's four arrow markers: neutral, accent, --ok, --bad. Which one an
// edge gets is a function of the handle it leaves from, so the drawing cannot
// disagree with the routing.
export const MARKERS = {
  neutral: 'var(--line2)',
  accent:  'var(--ac)',
  ok:      'var(--ok)',
  bad:     'var(--bad)',
} as const;
export type MarkerKind = keyof typeof MARKERS;

export const markerFor = (sourceHandle: string): MarkerKind => {
  switch (sourceHandle) {
    case 'true': case 'passed': case 'pass': return 'ok';
    case 'false': case 'failed': case 'reject': return 'bad';
    case 'start': case 'exit': return 'accent';
    default: return 'neutral';
  }
};

// solid-flow's node/edge shape, produced FROM the workflow graph rather than
// being it. The graph is the storage contract; this is a rendering of it.
export const toFlow = (g: WorkflowGraph) => ({
  nodes: g.nodes
    .filter((n) => n.kind in nodeTypes)
    .map((n) => ({ id: n.id, type: n.kind, position: { ...n.position }, data: { ...n.data } })),
  edges: g.edges.map((e) => ({
    id: e.id,
    source: e.source, sourceHandle: e.sourceHandle,
    target: e.target, targetHandle: e.targetHandle,
    animated: false,
    marker: markerFor(e.sourceHandle),
    dashed: Boolean(e.loopBack),
    style: e.loopBack
      ? { 'stroke-dasharray': '4 3', stroke: MARKERS[markerFor(e.sourceHandle)] }
      : { stroke: MARKERS[markerFor(e.sourceHandle)] },
    markerEnd: { type: MarkerType.ArrowClosed, color: MARKERS[markerFor(e.sourceHandle)] },
    label: e.sourceHandle === 'true' ? 'pass'
         : e.sourceHandle === 'false' ? 'fail · reset, fresh run, same session'
         : e.sourceHandle === 'start' ? 'approved'
         : e.sourceHandle === 'exit'  ? 'iteration >= 8'
         : undefined,
  })),
});

// The dashed grouping lives in GRAPH space, so it has to carry the same
// pan/zoom transform the nodes do.
//
// solid-flow exports ViewportPortal for exactly this and it does not work in
// 0.1.4: it mounts into `.solid-flow__viewport-portal`, an element this version
// never renders, so the selector returns null, Solid's Portal falls back to
// document.body, and the content lands in SCREEN space with no error raised.
// Reproduced — the group's parent chain came back `DIV -> BODY -> HTML`.
//
// So the transform is applied here instead, read from useViewport(). One line,
// and it is also the shape the dagre fallback would need, which is worth
// knowing: overlay content in graph space does not depend on the renderer.
const LoopGroups = (props: { groups: ReturnType<typeof loopGroups>; transformed?: boolean }) => {
  const viewport = useViewport();
  const outer = () => props.transformed === false
    ? ''
    : `translate(${viewport().x}px, ${viewport().y}px) scale(${viewport().zoom})`;
  return (
    <For each={props.groups}>{(g) => (
      <div class="wf-loop-group" data-loop={g.id}
           style={{ transform: `${outer()} translate(${g.x}px, ${g.y}px)`,
                    width: `${g.width}px`, height: `${g.height}px` }}>
        <span class="label">{g.label}</span>
      </div>
    )}</For>
  );
};

export function Canvas(props: { graph: WorkflowGraph; portalGroups?: boolean }) {
  const flow = createMemo(() => toFlow(props.graph));
  const [nodes, setNodes] = createNodeStore(flow().nodes as any);
  const [edges, setEdges] = createEdgeStore(flow().edges as any);
  const groups = createMemo(() => loopGroups(props.graph));

  return (
    <div class="canvas">
      <SolidFlow
        nodes={nodes} setNodes={setNodes}
        edges={edges} setEdges={setEdges}
        nodeTypes={nodeTypes as any}
        fitView
      >
        <Background />
        <Controls />
        {/* The dashed grouping is derived from its members, so it cannot drift
            out of sync with which nodes are in the loop.

            It goes through ViewportPortal, not next to the canvas. Drawn as a
            sibling it uses raw graph coordinates while the nodes carry the
            renderer's pan/zoom transform, so the rectangle detaches from the
            nodes it is supposed to enclose the moment anyone pans. The portal
            puts it inside the transformed layer, where its coordinates mean the
            same thing the nodes' do. */}
        <LoopGroups groups={groups()} transformed={props.portalGroups} />
      </SolidFlow>
    </div>
  );
}
