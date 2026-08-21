import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GRAPH_HEIGHT, GRAPH_WIDTH, WikiGraph } from '../../src/screens/wiki/WikiGraph'
import { GRAPH_CAPTION, useWikiGraph } from '../../src/data/wiki'
import { read, rules } from '../css'

const mount = () => render(() => <WikiGraph />)
const graph = useWikiGraph()

describe('wiki/graph', () => {
  it('is 258 by 132', () => {
    const { getByTestId } = mount()
    const svg = getByTestId('wiki-graph-svg')
    expect(svg.getAttribute('width')).toBe(String(GRAPH_WIDTH))
    expect(svg.getAttribute('height')).toBe(String(GRAPH_HEIGHT))
    expect(GRAPH_WIDTH).toBe(258)
    expect(GRAPH_HEIGHT).toBe(132)
  })

  it('draws one node per page and one edge per wikilink', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-graph').querySelectorAll('.graph-node').length).toBe(
      graph.nodes.length,
    )
    expect(getByTestId('wiki-graph').querySelectorAll('.graph-edge').length).toBe(
      graph.edges.length,
    )
  })

  it('gives the centre node 7px and the accent', () => {
    const { getByTestId } = mount()
    const centre = getByTestId('graph-node-w-clone').querySelector('circle')!
    expect(centre.getAttribute('r')).toBe('7')
    expect(centre.getAttribute('fill')).toBe('var(--ac)')
    expect(getByTestId('graph-node-w-clone').getAttribute('data-focal')).toBe('true')
  })

  it('gives the neighbours the blue and neutral grounds', () => {
    const { getByTestId } = mount()
    const fills = graph.nodes
      .filter((n) => !n.center)
      .map((n) => getByTestId(`graph-node-${n.id}`).querySelector('circle')!.getAttribute('fill'))
    expect(new Set(fills)).toEqual(new Set(['var(--blue-lit)', 'var(--sf3)']))
  })

  it('hairlines the edges', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.graph-edge')!.body,
    ).toContain('stroke: var(--line2)')
  })

  it('captions what the graph is', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-graph-caption').textContent).toBe(GRAPH_CAPTION)
    expect(GRAPH_CAPTION).toContain('Pages are nodes, wikilinks are edges')
  })

  it('labels the nodes in mono at 11px', () => {
    const { getByTestId } = mount()
    expect(getByTestId('wiki-graph').querySelectorAll('.graph-label').length).toBe(
      graph.nodes.length,
    )
    const body = rules(read('screens/screens.css')).find((r) => r.selector === '.graph-label')!.body
    expect(body).toContain('font-size: var(--t-micro)')
    expect(body).toContain('font-family: var(--fm)')
  })
})
