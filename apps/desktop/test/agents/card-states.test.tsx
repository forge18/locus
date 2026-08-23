import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { SessionCard } from '../../src/screens/automate/SessionCard'
import { useSessionDetail } from '../../src/data/sessions'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)!
const stuck = useSessionDetail('sd-weaver')!
const running = useSessionDetail('sd-tapestry')!

describe('agents/card-states', () => {
  it('marks the selected card and rings it in accent over --sf2', () => {
    const { getByTestId } = render(() => (
      <SessionCard session={running} selected onSelect={() => {}} />
    ))
    expect(getByTestId(`session-card-${running.id}`).getAttribute('aria-selected')).toBe('true')
    const body = rule(".session-card[aria-selected='true']").body
    expect(body).toContain('background: var(--surface-selected)')
    expect(body).toContain('box-shadow: var(--ring-sel-soft)')
  })

  it('hairlines a stuck card in red', () => {
    const { getByTestId } = render(() => (
      <SessionCard session={stuck} selected={false} onSelect={() => {}} />
    ))
    expect(getByTestId(`session-card-${stuck.id}`).className).toContain('session-card-stuck')
    expect(rule('.session-card-stuck').body).toContain('border-color: var(--status-danger)')
  })

  it('tints the chip by status — stuck red, waiting accent', () => {
    const waiting = useSessionDetail('sd-texere')!
    const s = render(() => <SessionCard session={stuck} selected={false} onSelect={() => {}} />)
    const w = render(() => <SessionCard session={waiting} selected={false} onSelect={() => {}} />)
    expect(s.getByTestId(`session-chip-${stuck.id}`).className).toContain('session-chip-stuck')
    expect(w.getByTestId(`session-chip-${waiting.id}`).className).toContain('session-chip-waiting')
  })

  it('gives each status its own dot colour', () => {
    const colours = ['running', 'waiting', 'idle', 'stuck', 'done'].map(
      (s) => rule(`.session-dot-${s}`).body,
    )
    expect(new Set(colours).size).toBe(colours.length)
  })

  it('marks exactly one card selected in the list', () => {
    const { getByTestId } = render(() => <AgentsView />)
    expect(getByTestId('session-list').querySelectorAll('[aria-selected="true"]').length).toBe(1)
  })
})
