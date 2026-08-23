import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop Commands', () => {
  it('renders authored configuration details', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="commands" />)
    expect(getByTestId('workshop-commands')).toBeTruthy()
  })
})
