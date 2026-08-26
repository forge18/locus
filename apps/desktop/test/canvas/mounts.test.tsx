import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/mounts', () => {
  it('mounts the shared Solid Flow canvas surface', () => {
    const { getByTestId } = render(() => <WorkflowView />)
    expect(getByTestId('wf-canvas')).toBeTruthy()
    expect(getByTestId('wf-solid-flow')).toBeTruthy()
  })
})
