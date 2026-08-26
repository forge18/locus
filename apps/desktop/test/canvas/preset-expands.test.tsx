import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/preset-expands', () => {
  it('expands Ralph into ordinary editable steps', async () => {
    const { getByTestId } = render(() => <WorkflowView />)
    await fireEvent.click(getByTestId('wf-preset-Ralph-loop'))
    const expanded = getByTestId('wf-preset-expanded')
    expect(expanded.getAttribute('data-preset')).toBe('Ralph loop')
    expect(expanded.textContent).toContain('ordinary nodes')
    expect(expanded.textContent).toContain('validate')
  })
})
