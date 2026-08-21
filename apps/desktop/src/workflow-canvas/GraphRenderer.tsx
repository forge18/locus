import { For, Show } from 'solid-js'
import type { JSX } from 'solid-js'

/**
 * One node/edge renderer, used by both the Workflow canvas and the Wiki graph.
 *
 * Two renderers that look alike are two renderers that will stop looking alike.
 * The wiki graph is this component repointed at pages and wikilinks — nothing
 * about it knows what a page is.
 */

export interface GraphNodeShape {
  id: string
  label: string
  x: number
  y: number
  /** Node radius in px. */
  r?: number
  /** A CSS colour, normally a token reference. */
  fill?: string
  /** Draws the accent ring a selected or focal node carries. */
  focal?: boolean
}

export interface GraphEdgeShape {
  from: string
  to: string
  label?: string
}

export interface GraphRendererProps {
  nodes: GraphNodeShape[]
  edges: GraphEdgeShape[]
  width: number
  height: number
  /** Rendered under the graph. */
  caption?: JSX.Element
  /** Draw node labels beside the dots. Off for a thumbnail. */
  showLabels?: boolean
  testId?: string
  onSelect?: (id: string) => void
}

const DEFAULT_R = 5

export function GraphRenderer(props: GraphRendererProps) {
  const byId = () => new Map(props.nodes.map((n) => [n.id, n]))

  return (
    <div class="graph" data-testid={props.testId ?? 'graph'}>
      <svg
        width={props.width}
        height={props.height}
        viewBox={`0 0 ${props.width} ${props.height}`}
        role="img"
        aria-label="Graph"
        data-testid={`${props.testId ?? 'graph'}-svg`}
      >
        <g class="graph-edges">
          <For each={props.edges}>
            {(edge) => {
              const a = byId().get(edge.from)
              const b = byId().get(edge.to)
              return (
                <Show when={a && b}>
                  <line
                    class="graph-edge"
                    x1={a!.x}
                    y1={a!.y}
                    x2={b!.x}
                    y2={b!.y}
                    data-from={edge.from}
                    data-to={edge.to}
                  />
                </Show>
              )
            }}
          </For>
        </g>
        <g class="graph-nodes">
          <For each={props.nodes}>
            {(node) => (
              <g
                class={['graph-node', node.focal ? 'graph-node-focal' : ''].filter(Boolean).join(' ')}
                data-testid={`graph-node-${node.id}`}
                data-focal={node.focal ? 'true' : undefined}
                onClick={props.onSelect ? () => props.onSelect!(node.id) : undefined}
              >
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={node.r ?? DEFAULT_R}
                  fill={node.fill ?? 'var(--sf3)'}
                />
                <Show when={props.showLabels}>
                  <text class="graph-label" x={node.x + (node.r ?? DEFAULT_R) + 4} y={node.y + 3}>
                    {node.label}
                  </text>
                </Show>
              </g>
            )}
          </For>
        </g>
      </svg>
      <Show when={props.caption}>
        <div class="graph-caption" data-testid={`${props.testId ?? 'graph'}-caption`}>
          {props.caption}
        </div>
      </Show>
    </div>
  )
}
