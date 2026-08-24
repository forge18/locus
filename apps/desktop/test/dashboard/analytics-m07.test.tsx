import { fireEvent, render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { AnalyticsView } from '../../src/screens/analytics/AnalyticsView'

describe('M0.7 Analytics', () => {
  it('keeps all four stat cards and redraws the selected measure', () => {
    const { getByTestId, getAllByText } = render(() => <AnalyticsView scope="all" />)
    expect(getByTestId('analytics-stat-spend')).toBeTruthy()
    expect(getByTestId('analytics-stat-tokens')).toBeTruthy()
    expect(getByTestId('analytics-stat-cache')).toBeTruthy()
    expect(getByTestId('analytics-stat-runs')).toBeTruthy()
    fireEvent.click(getByTestId('analytics-stat-tokens'))
    expect(getByTestId('analytics-trend').textContent).toContain('Selected measure: tokens')
    expect(getAllByText('Workflow').length).toBeGreaterThan(0)
  })

  it('renders Telemetry facets without a capture-source facet', () => {
    const { getByTestId, getByText, queryByText } = render(() => <AnalyticsView initialTab="telemetry" />)
    expect(getByTestId('analytics-telemetry')).toBeTruthy()
    expect(queryByText('capture source')).toBeNull()
    expect(getByText('permission_request')).toBeTruthy()
  })
})
