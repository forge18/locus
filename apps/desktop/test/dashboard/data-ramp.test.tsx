import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { V2DashboardView } from '../../src/screens/v2-dashboard'

describe('Dashboard data ramp', () => {
  it('marks token chart segments as data-ramp values', () => {
    const { getByTestId } = render(() => <V2DashboardView />)
    expect(getByTestId('v2-token-chart').getAttribute('data-data-ramp')).toBe('true')
  })
})
