import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { WAITING_NOTE, useSessionDetails } from '../../src/data/sessions'
import { read, rules } from '../css'

const waiting = useSessionDetails().find((s) => s.status === 'waiting')!
const idle = useSessionDetails().find((s) => s.status === 'idle')!

const mountAt = (id: string) => {
  const r = render(() => <AgentsView />)
  r.getByTestId(`session-card-${id}`).click()
  return r
}

describe('agents/waiting-not-idle', () => {
  it('states "Waiting ≠ idle." on a waiting session', () => {
    const { getByTestId } = mountAt(waiting.id)
    expect(getByTestId('waiting-note').textContent).toBe('Waiting ≠ idle.')
    expect(WAITING_NOTE).toBe('Waiting ≠ idle.')
  })

  it('carries the hourglass, not the moon', () => {
    const { getByTestId } = mountAt(waiting.id)
    expect(
      getByTestId('session-footer-waiting').querySelector('use')!.getAttribute('href'),
    ).toBe('#ph-hourglass-medium')
  })

  it('renders no footer at all on an idle session', () => {
    const { queryByTestId } = mountAt(idle.id)
    expect(queryByTestId('session-footer-waiting')).toBe(null)
    expect(queryByTestId('session-footer-stuck')).toBe(null)
  })

  it('gives the two statuses different dots and different chips', () => {
    const { getByTestId } = render(() => <AgentsView />)
    expect(getByTestId(`session-dot-${waiting.id}`).className).not.toBe(
      getByTestId(`session-dot-${idle.id}`).className,
    )
    expect(getByTestId(`session-chip-${waiting.id}`).className).not.toBe(
      getByTestId(`session-chip-${idle.id}`).className,
    )
  })

  it('grounds the waiting card on --sf rather than tinting it like a problem', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.waiting-card')!.body,
    ).toContain('background: var(--surface-raised)')
  })
})
