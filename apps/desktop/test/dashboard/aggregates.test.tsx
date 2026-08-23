import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2DashboardView } from '../../src/screens/v2-dashboard'

describe('Dashboard aggregates', () => {
  it('labels project, running, and model aggregate cards', () => {
    const { getByTestId } = render(() => <V2DashboardView />)
    expect(getByTestId('v2-dashboard').querySelectorAll('[data-dashboard-aggregate]')).toHaveLength(3)
  })
})
