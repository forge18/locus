import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RunningCount } from '../../src/shell/RunningCount'
import { useRunningCount } from '../../src/data/strip'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('shell/shell.css')).find((r) => r.selector === sel)

describe('shell/running-count', () => {
  it('reads "N running"', () => {
    const { getByTestId } = render(() => <RunningCount count={8} />)
    expect(getByTestId('running-count').textContent).toContain('8 running')
  })

  it('pulses the machine-working dot beside it', () => {
    const { getByTestId } = render(() => <RunningCount count={8} />)
    const dot = getByTestId('running-dot')
    expect(dot.className).toContain('pulse')
    expect(rule('.live-dot')!.body).toContain('background: var(--ac2)')
  })

  it('animates with the shared keyframe rather than one of its own', () => {
    expect(read('styles/motion.css')).toContain('.pulse { animation: pulse 2s')
  })

  it('counts agents only — a terminal you drive yourself is not one', () => {
    // The strip holds six cards; one of them is a shell.
    expect(useRunningCount()).toBe(5)
  })
})
