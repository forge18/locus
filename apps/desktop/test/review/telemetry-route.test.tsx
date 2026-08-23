import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { TelemetryView } from '../../src/screens/review/TelemetryView'

describe('Review telemetry route', () => {
  it('identifies the selected-project telemetry route', () => {
    const { getByTestId } = render(() => <TelemetryView />)
    expect(getByTestId('telemetry').getAttribute('data-desktop-route')).toBe('review-telemetry')
  })
})
