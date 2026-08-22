import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TelemetryView } from '../../src/screens/review/TelemetryView'
import { ACTION_NOTE, useActionRows } from '../../src/data/telemetry'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const mount = () => render(() => <TelemetryView />)

describe('telemetry/actions', () => {
  it('draws one row per verb', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-actions').querySelectorAll('.bar-row').length).toBe(
      useActionRows().length,
    )
  })

  it('gives the label a 132px column', () => {
    expect(rule('.bar-label').body).toContain('width: 132px')
  })

  it('draws a 7px track on a faint ground', () => {
    const body = rule('.bar-track').body
    expect(body).toContain('height: 7px')
    expect(body).toContain('background: rgba(238,242,246,.06)')
  })

  it('fills the track from the data ramp, and in --bad where the verb is a problem', () => {
    expect(rule('.bar-fill').body).toContain('background: var(--data-3)')
    expect(rule('.bar-fill-bad').body).toContain('background: var(--bad)')
    const { getByTestId } = mount()
    expect(getByTestId('action-tool_error').querySelector('.bar-fill')!.className).toContain(
      'bar-fill-bad',
    )
    expect(getByTestId('action-tool_call').querySelector('.bar-fill')!.className).not.toContain(
      'bar-fill-bad',
    )
  })

  it('right-aligns the count', () => {
    const { getByTestId } = mount()
    expect(getByTestId('action-tool_call').querySelector('.bar-count')!.textContent).toBe('50,796')
    expect(rule('.bar-count').body).toContain('text-align: right')
  })

  it('sizes each fill against the busiest verb', () => {
    const { getByTestId } = mount()
    const top = useActionRows()[0]
    expect(
      (getByTestId(`action-${top.verb}`).querySelector('.bar-fill') as HTMLElement).style.width,
    ).toBe('100%')
  })

  it('says the vocabulary is what every source normalizes to', () => {
    const { getByTestId } = mount()
    expect(getByTestId('tm-actions').textContent).toContain(ACTION_NOTE)
  })
})
