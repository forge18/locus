import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { GUARDRAIL_NOTE, HANDOFF_SUMMARY } from '../../src/data/sessions'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)

describe('agents/stuck-footer', () => {
  it('renders for the stuck session, which is the one selected first', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-footer-stuck')).toBeTruthy()
  })

  it('states the guardrail rule', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-footer-stuck').textContent).toContain(
      'kill & reassign after 3 stuck iterations',
    )
    expect(GUARDRAIL_NOTE).toBe('kill & reassign after 3 stuck iterations')
  })

  it('summarises what a handoff would carry — the expensive half is what was tried', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-footer-stuck').textContent).toContain(HANDOFF_SUMMARY)
    expect(HANDOFF_SUMMARY).toContain('already tried')
  })

  it('offers both answers: hand off, or let it run', () => {
    const { getByTestId } = mount()
    expect(getByTestId('guardrail-handoff').textContent).toBe('Hand off to reviewer@2')
    expect(getByTestId('guardrail-let-it-run').textContent).toBe('Let it run')
  })

  it('tints the card red and rings it', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.guardrail-card',
    )!.body
    expect(body).toContain('color-mix(in srgb, var(--bad) 8%, var(--sf))')
    expect(body).toContain('inset 0 0 0 1px var(--bad)')
  })
})
