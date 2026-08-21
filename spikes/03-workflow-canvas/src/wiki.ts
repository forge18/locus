// The Open question: can the wiki's graph view share this renderer?
//
// PLAN.md calls it "nearly free" — pages are nodes, [[wikilinks]] are edges —
// and assumes sharing makes it "a palette, not a subsystem". This is the
// smallest thing that tests the claim: parse wikilinks, produce the same node
// and edge shape the workflow canvas consumes.
export type WikiPage = { slug: string; title: string; body: string };

const LINK = /\[\[([^\]|]+)(?:\|[^\]]*)?\]\]/g;

export type WikiGraph = {
  nodes: { id: string; type: 'WikiPage'; position: { x: number; y: number }; data: { title: string } }[];
  edges: { id: string; source: string; target: string; sourceHandle: string; targetHandle: string }[];
};

export function wikiGraph(pages: WikiPage[]): WikiGraph {
  const known = new Set(pages.map((p) => p.slug));
  const nodes = pages.map((p, i) => ({
    id: p.slug, type: 'WikiPage' as const,
    // A ring, so the fixture has a layout without pulling in dagre. Real
    // placement is the graph view's problem, not this spike's.
    position: { x: 300 + 240 * Math.cos((i / pages.length) * 2 * Math.PI),
                y: 300 + 240 * Math.sin((i / pages.length) * 2 * Math.PI) },
    data: { title: p.title },
  }));
  const edges: WikiGraph['edges'] = [];
  for (const p of pages) {
    for (const m of p.body.matchAll(LINK)) {
      const target = m[1].trim();
      // A wikilink to a page that does not exist yet is not an edge. It is the
      // wiki's own affordance for "worth writing later".
      if (!known.has(target)) continue;
      edges.push({ id: `${p.slug}->${target}`, source: p.slug, target,
                   sourceHandle: 'out', targetHandle: 'in' });
    }
  }
  return { nodes, edges };
}
