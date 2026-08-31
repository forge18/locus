import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('workflow visual', () => {
  it('renders graph authoring without Governance state', () => {
    const { getByTestId, queryByTestId } = render(() => <WorkshopFixtureView fixture="workflows-visual" />)
    expect(getByTestId('workshop-workflows-visual')).toBeTruthy()
    expect(queryByTestId('workflow-governance-goal')).toBeNull()
  })
})
