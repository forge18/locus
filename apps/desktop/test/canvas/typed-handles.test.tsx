import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/typed-handles', () => {
  it('renders distinguishable named handles for routing nodes', () => {
    const { getByTestId } = render(() => <WorkflowView />)
    const handles = [...getByTestId('wf-node-n-cond').querySelectorAll('[data-handle-id]')]
      .map((handle) => handle.getAttribute('data-handle-id'))
    expect(handles).toEqual(expect.arrayContaining(['in', 'true', 'false']))
  })
})
