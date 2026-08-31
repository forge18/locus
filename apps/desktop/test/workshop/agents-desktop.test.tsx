import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/demo/WorkshopFixtureView'

describe('Workshop agents', () => {
  it('renders definition metadata', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="agents" />)
    expect(getByTestId('workshop-agents')).toBeTruthy()
  })
})
