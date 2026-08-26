import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { AT_A_GLANCE_METRICS } from '../../src/data/analytics'
import { AnalyticsView } from '../../src/screens/analytics/AnalyticsView'

describe('telemetry/from-core', () => {
  it('renders the full metric set beside facets and normalized-event search', () => {
    const { getByTestId, getByPlaceholderText } = render(() => <AnalyticsView initialTab="telemetry" />)
    expect(getByTestId('analytics-facets')).toBeTruthy()
    expect(getByPlaceholderText('BM25 search over the normalized event log')).toBeTruthy()
    expect(getByTestId('telemetry-metrics').querySelectorAll('[data-metric]').length).toBe(AT_A_GLANCE_METRICS.length)
  })
})
