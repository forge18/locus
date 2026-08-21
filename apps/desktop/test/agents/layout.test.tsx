import { describe, expect, it } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { AgentsView } from '../../src/screens/automate/AgentsView'
import { read, rules } from '../css'

const mount = () => render(() => <AgentsView />)

describe('agents/layout', () => {
  it('is a two-pane split', () => {
    const { getByTestId } = mount()
    expect(getByTestId('session-list')).toBeTruthy()
    expect(getByTestId('transcript-pane')).toBeTruthy()
  })

  it('starts the list at 356px', () => {
    const { getByTestId } = mount()
    expect((getByTestId('session-list') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('356px')
  })

  it('lets the transcript take the rest', () => {
    const body = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.transcript-pane',
    )!.body
    expect(body).toContain('flex: 1')
    expect(body).toContain('min-width: 0')
  })

  it('hairlines the seam', () => {
    expect(
      rules(read('screens/screens.css')).find((r) => r.selector === '.session-list')!.body,
    ).toContain('border-right: 1px solid var(--line)')
  })

  it('is resizable, because the drawn width is a default', () => {
    const { getByTestId } = mount()
    fireEvent.pointerDown(getByTestId('session-list-handle'), { clientX: 0 })
    fireEvent.pointerMove(document, { clientX: 44 })
    expect((getByTestId('session-list') as HTMLElement).style.getPropertyValue('--pane-w')).toBe('400px')
    fireEvent.pointerUp(document)
  })
})
