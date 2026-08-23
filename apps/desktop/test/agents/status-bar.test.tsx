import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { PTY_NOTE, useSessionDetails } from '../../src/data/sessions'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)

describe('agents/status-bar', () => {
  it('states that the PTY is attached from the host', () => {
    const { getByTestId } = mount()
    expect(getByTestId('pty-note').textContent).toBe(
      'PTY attached from the host · one session per terminal',
    )
    expect(PTY_NOTE).toContain('one session per terminal')
  })

  it('shows the run id on the right', () => {
    const { getByTestId } = mount()
    expect(getByTestId('run-id').textContent).toBe(useSessionDetails()[0].runId)
  })

  it('moves the run id with the selection', () => {
    const { getByTestId } = mount()
    const other = useSessionDetails()[1]
    getByTestId(`session-card-${other.id}`).click()
    expect(getByTestId('run-id').textContent).toBe(other.runId)
  })

  it('is mono on the deep ground', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.session-status-bar',
    )!.body
    expect(body).toContain('font-family: var(--fm)')
    expect(body).toContain('background: var(--surface-chrome)')
  })

  it('sits at the very foot, below any conditional footer', () => {
    const { getByTestId } = mount()
    const bar = getByTestId('session-status-bar')
    const footer = getByTestId('session-footer-stuck')
    expect(
      footer.compareDocumentPosition(bar) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
