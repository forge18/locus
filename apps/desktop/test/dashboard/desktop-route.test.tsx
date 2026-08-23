import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopDashboardView } from '../../src/screens/desktop-dashboard'

describe('Dashboard desktop route', () => {
  it('identifies the global dashboard fixture route', () => {
    const { getByTestId } = render(() => <DesktopDashboardView />)
    expect(getByTestId('desktop-dashboard').getAttribute('data-desktop-route')).toBe('dashboard')
  })
})
