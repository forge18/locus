import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { parse } from '../../src/nav'
import { read } from '../css'

describe('agents/detach-is-a-window', () => {
  it('hands the caller a locator, which is what a second window opens on', () => {
    const detached: string[] = []
    const { getByTestId } = render(() => <AgentsView onDetach={(l) => detached.push(l)} />)
    getByTestId('transcript-detach').click()
    expect(detached.length).toBe(1)
    expect(parse(detached[0]).kind).toBe('session')
  })

  it('opens no webview of its own — the screen has no such call', () => {
    const source = read('screens/automate/AgentsView.tsx')
    // The word appears in the comment explaining why not; what must be absent is
    // any code that would actually make one.
    expect(source).not.toMatch(/createElement\(['"]webview|<webview[\s>]|new Webview/i)
  })

  it('states in the source that a detached pane is a window, not a second webview', () => {
    const source = read('screens/automate/AgentsView.tsx')
    expect(source).toContain('Never a second webview in one window')
  })

  it('leaves the session in the list after detaching', () => {
    const { getByTestId } = render(() => <AgentsView onDetach={() => {}} />)
    const before = getByTestId('session-list').querySelectorAll('.session-card').length
    getByTestId('transcript-detach').click()
    expect(getByTestId('session-list').querySelectorAll('.session-card').length).toBe(before)
  })

  it('does nothing at all when the host offers no detach', () => {
    const { getByTestId } = render(() => <AgentsView />)
    expect(() => getByTestId('transcript-detach').click()).not.toThrow()
  })
})
