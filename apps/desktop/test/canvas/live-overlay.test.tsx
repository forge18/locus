import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/live-overlay', () => {
  it('paints the node overlay from normalized workflow events', () => {
    const { getByTestId } = render(() => <WorkflowView />)
    const overlay = getByTestId('wf-live-overlay')
    expect(overlay.getAttribute('data-event-source')).toBe('workflow-events')
    expect(overlay.textContent).toContain('normalized events')
  })
})
