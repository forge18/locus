import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { TelemetryView } from '../../src/screens/review/TelemetryView'

describe('Review telemetry filters', () => {
  it('labels filters, facets, and tool-error evidence', () => {
    const { getByTestId } = render(() => <TelemetryView />)
    expect(getByTestId('telemetry').getAttribute('data-filter-evidence')).toBe('available')
  })
})
