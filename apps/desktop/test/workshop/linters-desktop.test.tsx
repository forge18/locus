import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop Linters', () => {
  it('renders authored configuration details', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="linters" />)
    expect(getByTestId('workshop-linters')).toBeTruthy()
  })
})
