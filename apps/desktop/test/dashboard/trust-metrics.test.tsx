import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2DashboardView } from '../../src/screens/v2-dashboard'

describe('Dashboard trust metrics', () => {
  it('shows steer-versus-review and review debt', () => {
    const { getByTestId } = render(() => <V2DashboardView />)
    const metrics = getByTestId('v2-dashboard-counters').textContent
    expect(metrics).toContain('Steer vs review')
    expect(metrics).toContain('Review debt')
  })
})
