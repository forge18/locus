import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('workflow governance', () => {
  it('renders goal, guardrails, and success criteria', () => {
    const { getByTestId, getByText } = render(() => <WorkshopFixtureView fixture="workflows-governance" />)
    expect(getByTestId('workflow-governance-goal')).toBeTruthy()
    expect(getByTestId('workflow-success-criteria')).toBeTruthy()
    expect(getByText('Preserve branches')).toBeTruthy()
  })
})
