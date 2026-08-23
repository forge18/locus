import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { useTelemetryMetrics } from '../../src/data/telemetry'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)

describe('telemetry/metrics', () => {
  it('shows four metric cards beside the sparkline', () => {
    const { getByTestId } = mount()
    const cards = getByTestId('tm-metrics').querySelectorAll('.metric-card')
    expect(cards.length).toBe(5)
    expect(useTelemetryMetrics().length).toBe(4)
  })

  it('names them Sessions, Events, Tool errors and Output tokens', () => {
    expect(useTelemetryMetrics().map((m) => m.label)).toEqual([
      'Sessions',
      'Events',
      'Tool errors',
      'Output tokens',
    ])
  })

  it('draws Tool errors in --bad with a red hairline, and only it', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-metric-tool-errors').getAttribute('data-bad')).toBe('true')
    expect(getByTestId('tm-metrics').querySelectorAll('[data-bad="true"]').length).toBe(1)
    expect(rule('.tm-metric-bad').body).toContain('border-color: var(--status-danger)')
    expect(rule('.tm-metric-bad .metric-numeral').body).toContain('color: var(--status-danger)')
  })

  it('shows the unit suffix where there is one', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-metric-output-tokens').textContent).toContain('77.46')
    expect(getByTestId('tm-metric-output-tokens').querySelector('.metric-unit')!.textContent).toBe('M')
  })

  it('reflows the cards rather than squeezing five into one row at any width', () => {
    expect(rule('.tm-metrics').body).toContain('repeat(auto-fit, minmax(170px, 1fr))')
  })
})
