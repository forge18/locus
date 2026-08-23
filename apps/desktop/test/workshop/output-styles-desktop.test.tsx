import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop OutputStyles', () => {
  it('renders authored configuration details', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="output-styles" />)
    expect(getByTestId('workshop-output-styles')).toBeTruthy()
  })
})
