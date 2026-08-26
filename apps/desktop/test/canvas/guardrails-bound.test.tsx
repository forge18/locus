import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { useGuardrails } from '../../src/data/workflow'
import { WorkflowView } from '../../src/screens/workshop/WorkflowView'

describe('canvas/guardrails-bound', () => {
  it('renders every workflow guardrail from the shared config', () => {
    const { getByTestId } = render(() => <WorkflowView />)
    for (const guardrail of useGuardrails()) expect(getByTestId(`guardrail-${guardrail.key}`)).toBeTruthy()
  })
})
