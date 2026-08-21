import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { MetricCard } from '../../src/screens/status/MetricCard'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)

const METRIC = {
  label: 'Verify pass',
  value: '71',
  unit: '%',
  note: 'last 24h',
  attention: false,
  badNote: null,
}

describe('status/metric-card', () => {
  it('sets the numeral at 34px/500', () => {
    const { getByTestId } = render(() => <MetricCard metric={METRIC} />)
    expect(getByTestId('metric-numeral').textContent).toBe('71')
    expect(rule('.metric-numeral')!.body).toContain('font-size: var(--t-metric-lg)')
    expect(rule('.metric-numeral')!.body).toContain('font-weight: 500')
  })

  it('sets the unit suffix at 19px in --mu', () => {
    const { getByTestId } = render(() => <MetricCard metric={METRIC} />)
    expect(getByTestId('metric-unit').textContent).toBe('%')
    expect(rule('.metric-unit')!.body).toContain('font-size: var(--t-title)')
    expect(rule('.metric-unit')!.body).toContain('color: var(--mu)')
  })

  it('sets the label at 13px uppercase with .1em tracking', () => {
    const body = rule('.metric-label')!.body
    expect(body).toContain('font-size: var(--t-meta)')
    expect(body).toContain('text-transform: uppercase')
    expect(body).toContain('letter-spacing: .1em')
  })

  it('shows the note under the numeral', () => {
    const { getByTestId } = render(() => <MetricCard metric={METRIC} />)
    expect(getByTestId('metric-note').textContent).toBe('last 24h')
  })

  it('puts a bad note in --bad instead of --mu2', () => {
    const { getByTestId } = render(() => (
      <MetricCard
        metric={{ ...METRIC, label: 'Guardrail trips', note: null, badNote: '1 kill & reassign' }}
      />
    ))
    expect(getByTestId('metric-note').className).toContain('metric-note-bad')
    expect(rule('.metric-note-bad')!.body).toContain('color: var(--bad)')
  })

  it('omits the unit when there is none', () => {
    const { queryByTestId } = render(() => <MetricCard metric={{ ...METRIC, unit: null }} />)
    expect(queryByTestId('metric-unit')).toBe(null)
  })
})
