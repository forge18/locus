import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2DashboardView } from '../../src/screens/v2-dashboard'

describe('Dashboard v2 route', () => {
  it('identifies the global dashboard fixture route', () => {
    const { getByTestId } = render(() => <V2DashboardView />)
    expect(getByTestId('v2-dashboard').getAttribute('data-v2-route')).toBe('dashboard')
  })
})
