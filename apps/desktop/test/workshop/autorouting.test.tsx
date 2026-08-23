import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop autorouting', () => {
  it('renders six routing bands and fallback', () => {
    const { getAllByTestId, getByTestId } = render(() => <WorkshopFixtureView fixture="harnesses" />)
    expect(getAllByTestId(/autoroute-band-/)).toHaveLength(6)
    expect(getByTestId('autoroute-fallback').textContent).toContain('falls upward')
  })
})
