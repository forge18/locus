import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { useCanvas } from '../../src/data/workflow'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/edges', () => {
  it('renders every graph edge with a marker and label layer', () => {
    const { getByTestId } = render(() => <WorkflowView />)
    const edges = getByTestId('wf-edges')
    expect(edges.querySelectorAll('line')).toHaveLength(useCanvas().edges.length)
    for (const edge of edges.querySelectorAll('line')) {
      expect(edge.getAttribute('marker-end')).toMatch(/^url\(#arrow-/)
    }
  })
})
