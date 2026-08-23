import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopDashboardView } from '../../src/screens/desktop-dashboard'

describe('Dashboard trust metrics', () => {
  it('shows steer-versus-review and review debt', () => {
    const { getByTestId } = render(() => <DesktopDashboardView />)
    const metrics = getByTestId('desktop-dashboard-counters').textContent
    expect(metrics).toContain('Steer vs review')
    expect(metrics).toContain('Review debt')
  })
})
