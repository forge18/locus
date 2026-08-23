import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { StatusView } from '../../src/screens/status/StatusView'
import { useStatusMetrics } from '../../src/data/status'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = () => render(() => <StatusView />)

describe('status/six-metrics', () => {
  it('shows six cards in a grid that reflows rather than squeezing', () => {
    const { getByTestId } = mount()
    expect(getByTestId('status-metrics').querySelectorAll('.metric-card').length).toBe(6)
    expect(rule('.status-metrics')!.body).toContain('repeat(auto-fit, minmax(150px, 1fr))')
  })

  it('shows the six the design names, in order', () => {
    expect(useStatusMetrics().map((m) => m.label)).toEqual([
      'Running',
      'Waiting on me',
      'Verify pass',
      'Cache read',
      'Tokens today',
      'Guardrail trips',
    ])
  })

  it('gives "Waiting on me" the accent card, and only it', () => {
    const { getByTestId } = mount()
    const attention = getByTestId('status-metrics').querySelectorAll('[data-attention="true"]')
    expect(attention.length).toBe(1)
    expect(getByTestId('metric-waiting-on-me').getAttribute('data-attention')).toBe('true')
  })

  it('paints the accent card --sf2 with the accent ring, label and numeral', () => {
    expect(rule('.metric-card-attention')!.body).toContain('background: var(--surface-selected)')
    expect(rule('.metric-card-attention')!.body).toContain('box-shadow: var(--ring-sel)')
    const accented = rule('.metric-card-attention .metric-label,\n.metric-card-attention .metric-numeral')!
    expect(accented.body).toContain('color: var(--action-attention)')
  })

  it('carries "oldest 26m" on it — the age is what makes it urgent', () => {
    const { getByTestId } = mount()
    expect(getByTestId('metric-waiting-on-me').textContent).toContain('oldest 26m')
  })

  it('notes the panes/strip split on Running, and the kill & reassign in --bad', () => {
    const { getByTestId } = mount()
    expect(getByTestId('metric-running').textContent).toContain('4 panes · 4 strip')
    expect(getByTestId('metric-guardrail-trips').querySelector('.metric-note-bad')!.textContent).toBe(
      '1 kill & reassign',
    )
  })
})
