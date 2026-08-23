import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { DesktopDashboardView } from '../../src/screens/desktop-dashboard'

describe('Dashboard aggregates', () => {
  it('labels project, running, and model aggregate cards', () => {
    const { getByTestId } = render(() => <DesktopDashboardView />)
    expect(getByTestId('desktop-dashboard').querySelectorAll('[data-dashboard-aggregate]')).toHaveLength(3)
  })
})
