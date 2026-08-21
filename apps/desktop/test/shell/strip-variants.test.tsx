import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Strip } from '../../src/shell/Strip'
import type { StripCard } from '../../src/data/strip'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('shell/shell.css')).find((r) => r.selector === sel)

const STUCK: StripCard = {
  id: 'st-stuck', kind: 'agent', project: 'weaver', agent: 'builder@4', role: 'builder',
  status: 'stuck', tool: 'run_command', tokens: 102_300, idleMinutes: 14,
}

const SHELL: StripCard = {
  id: 'st-shell', kind: 'shell', project: 'tapestry', agent: null, role: null,
  status: null, tool: null, tokens: null, idleMinutes: 1,
}

describe('shell/strip-variants', () => {
  it('borders a stuck card in --bad', () => {
    const { getByTestId } = render(() => <Strip cards={[STUCK]} />)
    expect(getByTestId('strip-card-st-stuck').className).toContain('strip-card-stuck')
    expect(rule('.strip-card-stuck')!.body).toContain('border-color: var(--bad)')
  })

  it('dims a shell card', () => {
    const { getByTestId } = render(() => <Strip cards={[SHELL]} />)
    expect(getByTestId('strip-card-st-shell').className).toContain('strip-card-shell')
    expect(rule('.strip-card-shell')!.body).toMatch(/opacity:\s*\.6/)
  })

  it('marks a shell card with the terminal glyph', () => {
    const { getByTestId } = render(() => <Strip cards={[SHELL]} />)
    expect(getByTestId('strip-card-st-shell').querySelector('use')!.getAttribute('href')).toBe(
      '#ph-terminal-window',
    )
  })

  it('reads "no agent · no cost" on a shell, because it is neither', () => {
    const { getByTestId } = render(() => <Strip cards={[SHELL]} />)
    expect(getByTestId('strip-card-st-shell').querySelector('.strip-card-bottom')!.textContent).toBe(
      'no agent · no cost',
    )
  })

  it('leaves an ordinary running card with the plain hairline', () => {
    const running: StripCard = { ...STUCK, id: 'st-ok', status: 'running' }
    const { getByTestId } = render(() => <Strip cards={[running]} />)
    const cls = getByTestId('strip-card-st-ok').className
    expect(cls).not.toContain('strip-card-stuck')
    expect(cls).not.toContain('strip-card-shell')
  })
})
