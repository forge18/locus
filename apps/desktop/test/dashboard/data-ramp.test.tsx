import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopDashboardView } from '../../src/screens/desktop-dashboard'

describe('Dashboard data ramp', () => {
  it('marks token chart segments as data-ramp values', () => {
    const { getByTestId } = render(() => <DesktopDashboardView />)
    expect(getByTestId('desktop-token-chart').getAttribute('data-data-ramp')).toBe('true')
  })
})
