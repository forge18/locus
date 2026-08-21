import { describe, expect, it } from 'vitest';
import { render } from '@solidjs/testing-library';
import { SolidFlow, Handle, Position, createNodeStore, createEdgeStore } from '@dschz/solid-flow';
import { wikiGraph } from '../src/wiki';

// The Open question, answered rather than assumed: PLAN.md says the wiki graph
// view is nearly free because it reuses this renderer.
const pages = [
  { slug: 'containers',  title: 'Containers',  body: 'One per run. See [[credentials]] and [[git-model]].' },
  { slug: 'credentials', title: 'Credentials', body: 'The host proxy. Related: [[containers]].' },
  { slug: 'git-model',   title: 'Git model',   body: 'A local remote. Also [[not-yet-written]].' },
];

const WikiNode = (props: any) => (
  <div class="wiki-node" data-page={props.id}>
    <Handle type="target" position={Position.Left} id="in" />
    {props.data.title}
    <Handle type="source" position={Position.Right} id="out" />
  </div>
);

describe('the same renderer, pointed at a wikilink graph', () => {
  it('turns pages into nodes and resolved wikilinks into edges', () => {
    const g = wikiGraph(pages);
    expect(g.nodes.map((n) => n.id).sort()).toEqual(['containers', 'credentials', 'git-model']);
    expect(g.edges.map((e) => e.id).sort()).toEqual([
      'containers->credentials', 'containers->git-model', 'credentials->containers',
    ]);
    // A link to a page nobody has written is not an edge.
    expect(g.edges.some((e) => e.target === 'not-yet-written')).toBe(false);
  });

  it('renders through the SAME <SolidFlow> with only a different node component', () => {
    // If this needs anything the workflow canvas does not, "a palette, not a
    // subsystem" is wrong.
    const g = wikiGraph(pages);
    const [nodes, setNodes] = createNodeStore(g.nodes as any);
    const [edges, setEdges] = createEdgeStore(g.edges as any);
    const { container } = render(() => (
      <SolidFlow nodes={nodes} setNodes={setNodes} edges={edges} setEdges={setEdges}
                 nodeTypes={{ WikiPage: WikiNode } as any} fitView />
    ));
    expect(container.querySelectorAll('.wiki-node').length).toBe(3);
    expect(container.querySelector('.wiki-node[data-page="credentials"]')!.textContent)
      .toContain('Credentials');
  });
});
