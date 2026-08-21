import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { useSessionDetails } from '../../src/data/sessions'

const mount = () => render(() => <AgentsView />)

describe('agents/footer-exclusive', () => {
  it('shows at most one conditional footer at a time', () => {
    const { getByTestId, queryByTestId } = mount()
    for (const session of useSessionDetails()) {
      getByTestId(`session-card-${session.id}`).click()
      const shown = ['session-footer-stuck', 'session-footer-waiting'].filter(
        (id) => queryByTestId(id) !== null,
      )
      expect(shown.length, session.id).toBeLessThanOrEqual(1)
    }
  })

  it('shows the stuck footer exactly when the session is stuck', () => {
    const { getByTestId, queryByTestId } = mount()
    for (const session of useSessionDetails()) {
      getByTestId(`session-card-${session.id}`).click()
      expect(queryByTestId('session-footer-stuck') !== null, session.id).toBe(
        session.status === 'stuck',
      )
    }
  })

  it('shows the waiting footer exactly when the session is waiting', () => {
    const { getByTestId, queryByTestId } = mount()
    for (const session of useSessionDetails()) {
      getByTestId(`session-card-${session.id}`).click()
      expect(queryByTestId('session-footer-waiting') !== null, session.id).toBe(
        session.status === 'waiting',
      )
    }
  })

  it('shows neither on a running session', () => {
    const running = useSessionDetails().find((s) => s.status === 'running')!
    const { getByTestId, queryByTestId } = mount()
    getByTestId(`session-card-${running.id}`).click()
    expect(queryByTestId('session-footer-stuck')).toBe(null)
    expect(queryByTestId('session-footer-waiting')).toBe(null)
  })

  it('keeps the status bar whatever the footer does', () => {
    const { getByTestId } = mount()
    for (const session of useSessionDetails()) {
      getByTestId(`session-card-${session.id}`).click()
      expect(getByTestId('session-status-bar'), session.id).toBeTruthy()
    }
  })
})
