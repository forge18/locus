import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunsByHour } from '../../src/screens/status/RunsByHour'
import { useRunsByHour } from '../../src/data/status'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = () => render(() => <RunsByHour hours={useRunsByHour()} />)

describe('status/runs-by-hour', () => {
  it('draws twelve bars', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hours').querySelectorAll('.hour-bar').length).toBe(12)
  })

  it('is 118px tall with 5px gaps', () => {
    const body = rule('.hours')!.body
    expect(body).toContain('height: 118px')
    expect(body).toContain('gap: 5px')
  })

  it('stacks bottom-up', () => {
    expect(rule('.hour-bar')!.body).toContain('flex-direction: column-reverse')
  })

  it('colors passed accent, failed --bad and aborted --blue-lit', () => {
    expect(rule('.hour-seg-passed')!.body).toContain('background: var(--ac)')
    expect(rule('.hour-seg-failed')!.body).toContain('background: var(--bad)')
    expect(rule('.hour-seg-aborted')!.body).toContain('background: var(--blue-lit)')
  })

  it('puts the three states in every bar, passed first from the bottom', () => {
    const { getByTestId } = mount()
    const segs = [...getByTestId('hour-08').children].map((c) => c.getAttribute('data-segment'))
    expect(segs).toEqual(['passed', 'failed', 'aborted'])
  })

  it('sizes each bar against the busiest hour', () => {
    const { getByTestId } = mount()
    const hours = useRunsByHour()
    const max = Math.max(...hours.map((h) => h.passed + h.failed + h.aborted))
    const busiest = hours.find((h) => h.passed + h.failed + h.aborted === max)!
    expect((getByTestId(`hour-${busiest.hour}`) as HTMLElement).style.height).toBe('100%')
  })

  it('labels the axis in mono', () => {
    const { getByTestId } = mount()
    expect(getByTestId('hour-axis').textContent).toContain('08')
    expect(rule('.hour-axis')!.body).toContain('font-family: var(--fm)')
  })
})
