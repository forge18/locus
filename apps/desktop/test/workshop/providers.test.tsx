import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { WorkshopFixtureView } from '../../src/screens/workshop/WorkshopFixtureView'

describe('Workshop providers', () => {
  it('renders provider status and masked authentication form', () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="providers" />)
    expect(getByTestId('provider-secret').textContent).toMatch(/^•+$/)
    expect(getByTestId('provider-verification').textContent).toContain('verified')
  })
})
