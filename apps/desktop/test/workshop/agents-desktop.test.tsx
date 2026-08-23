import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop agents', () => {
  it('renders definition metadata', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="agents" />)
    expect(getByTestId('workshop-agents')).toBeTruthy()
  })
})
