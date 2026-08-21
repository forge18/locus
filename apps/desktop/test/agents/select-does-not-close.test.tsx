import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { useSessionDetails } from '../../src/data/sessions'

const mount = () => render(() => <AgentsView />)
const sessions = useSessionDetails()

describe('agents/select-does-not-close', () => {
  it('swaps the transcript', () => {
    const { getByTestId } = mount()
    expect(getByTestId('transcript').textContent).toContain('weaver')
    getByTestId(`session-card-${sessions[3].id}`).click()
    expect(getByTestId('transcript').textContent).toContain(sessions[3].transcript[0].text)
  })

  it('swaps the header and the status chip with it', () => {
    const { getByTestId } = mount()
    const other = sessions.find((s) => s.status === 'running')!
    getByTestId(`session-card-${other.id}`).click()
    expect(getByTestId('transcript-head').textContent).toContain(other.project)
    expect(getByTestId('transcript-status').textContent).toBe('running')
  })

  it('swaps the conditional footer with it', () => {
    const { getByTestId, queryByTestId } = mount()
    expect(queryByTestId('session-footer-stuck')).not.toBe(null)
    const running = sessions.find((s) => s.status === 'running')!
    getByTestId(`session-card-${running.id}`).click()
    expect(queryByTestId('session-footer-stuck')).toBe(null)
  })

  it('leaves every other session in the list', () => {
    const { getByTestId } = mount()
    getByTestId(`session-card-${sessions[2].id}`).click()
    for (const session of sessions) {
      expect(getByTestId(`session-card-${session.id}`), session.id).toBeTruthy()
    }
  })

  it('leaves the others running — their status does not change', () => {
    const { getByTestId } = mount()
    const before = sessions.map((s) => getByTestId(`session-chip-${s.id}`).textContent)
    getByTestId(`session-card-${sessions[2].id}`).click()
    const after = sessions.map((s) => getByTestId(`session-chip-${s.id}`).textContent)
    expect(after).toEqual(before)
  })

  it('moves the selection ring, and only the ring', () => {
    const { getByTestId } = mount()
    getByTestId(`session-card-${sessions[1].id}`).click()
    expect(getByTestId(`session-card-${sessions[1].id}`).getAttribute('aria-selected')).toBe('true')
    expect(getByTestId(`session-card-${sessions[0].id}`).getAttribute('aria-selected')).toBe('false')
  })
})
