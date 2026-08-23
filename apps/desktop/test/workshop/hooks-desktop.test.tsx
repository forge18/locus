import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop Hooks', () => {
  it('renders authored configuration details', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="hooks" />)
    expect(getByTestId('workshop-hooks')).toBeTruthy()
  })
})
