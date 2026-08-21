import { GraphRenderer } from '../../workflow-canvas/GraphRenderer'
import { GRAPH_CAPTION, useWikiGraph } from '../../data/wiki'

export const GRAPH_WIDTH = 258
export const GRAPH_HEIGHT = 132

/**
 * The same renderer the Workflow canvas uses, repointed at pages and wikilinks.
 * Nothing in `GraphRenderer` knows what a page is, which is what keeps one
 * renderer from quietly becoming two.
 */
export function WikiGraph() {
  const graph = useWikiGraph()

  return (
    <GraphRenderer
      testId="wiki-graph"
      width={GRAPH_WIDTH}
      height={GRAPH_HEIGHT}
      nodes={graph.nodes.map((n) => ({
        id: n.id,
        label: n.label,
        x: n.x,
        y: n.y,
        r: n.center ? 7 : 5,
        fill: n.center ? 'var(--ac)' : n.id.length % 2 === 0 ? 'var(--blue-lit)' : 'var(--sf3)',
        focal: n.center,
      }))}
      edges={graph.edges}
      showLabels
      caption={GRAPH_CAPTION}
    />
  )
}
