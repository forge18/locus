import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Strip } from '../../src/shell/Strip'
import type { StripCard } from '../../src/data/strip'

const AGENT: StripCard = {
  id: 'st-a', kind: 'agent', project: 'tapestry', agent: 'builder@4', role: 'builder',
  status: 'running', tool: 'edit_file', tokens: 41_200, idleMinutes: 0,
}

const UNKNOWN: StripCard = { ...AGENT, id: 'st-b', tokens: null, tool: null, status: 'waiting' }

describe('shell/strip-card', () => {
  it('puts project · agent · role on top', () => {
    const { getByTestId } = render(() => <Strip cards={[AGENT]} />)
    const card = getByTestId('strip-card-st-a')
    expect(card.querySelector('.strip-card-top')!.textContent).toBe('tapestry · builder@4 · builder')
  })

  it('puts status · tool · tokens underneath', () => {
    const { getByTestId } = render(() => <Strip cards={[AGENT]} />)
    expect(getByTestId('strip-card-st-a').querySelector('.strip-card-bottom')!.textContent).toBe(
      'running · edit_file · 41.2k',
    )
  })

  it('reads unknown, not zero, where the harness reports no usage', () => {
    const { getByTestId } = render(() => <Strip cards={[UNKNOWN]} />)
    const bottom = getByTestId('strip-card-st-b').querySelector('.strip-card-bottom')!.textContent!
    expect(bottom).toContain('unknown')
    expect(bottom).not.toContain('0 ')
  })

  it('says "no tool" rather than leaving a gap when the agent is between tools', () => {
    const { getByTestId } = render(() => <Strip cards={[UNKNOWN]} />)
    expect(getByTestId('strip-card-st-b').textContent).toContain('no tool')
  })

  it('exposes kind and status to the DOM, so state is testable and not just painted', () => {
    const { getByTestId } = render(() => <Strip cards={[AGENT]} />)
    expect(getByTestId('strip-card-st-a').getAttribute('data-kind')).toBe('agent')
    expect(getByTestId('strip-card-st-a').getAttribute('data-status')).toBe('running')
  })
})
