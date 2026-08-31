import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import InteractView from '../../src/screens/interact/InteractView'

// The rail's per-row delete and the panel's Discard are the same operation
// (.specs/interact-sessions/spec.md §Sessions rail), and that operation is
// offered only while a session is open — discarding a promoted session is
// refused, and a discarded session is terminal. So the card `×` renders only
// where discard is valid instead of failing silently on the other states.

const cardFor = (container: HTMLElement, name: string) =>
  Array.from(container.querySelectorAll<HTMLElement>('.interact-session-card')).find(
    (card) => card.querySelector('strong')?.textContent === name,
  )!

const discardControl = (card: HTMLElement) =>
  card.querySelector<HTMLButtonElement>('button[aria-label^="Discard"]')

describe('interact/discard', () => {
  it('renders the per-card discard control only on open sessions', () => {
    const { container } = render(() => <InteractView />)

    const open = cardFor(container, 'Try the notification path')
    expect(open.getAttribute('data-state')).toBe('open')
    expect(discardControl(open)?.getAttribute('aria-label')).toBe(
      'Discard Try the notification path',
    )
    expect(discardControl(cardFor(container, 'Review parser behavior'))).toBe(null)
    expect(discardControl(cardFor(container, 'Discarded experiment'))).toBe(null)
    expect(container.querySelectorAll('button[aria-label^="Discard"]')).toHaveLength(1)
  })

  it('discards an open session from its card and retires the ending controls', () => {
    const { container } = render(() => <InteractView />)

    expect(container.querySelector('.interact-actions')).not.toBe(null)
    discardControl(cardFor(container, 'Try the notification path'))!.click()

    // Discard swaps the session object, so <For> replaces the row node — re-query
    // the live card rather than holding the pre-click reference.
    const discarded = cardFor(container, 'Try the notification path')
    expect(discarded.getAttribute('data-state')).toBe('discarded')
    expect(discarded.textContent).toContain('discarded')
    expect(discardControl(discarded)).toBe(null)
    expect(container.querySelector('.interact-actions')).toBe(null)
  })

  it('offers no ending controls over promoted and discarded sessions', () => {
    const { container } = render(() => <InteractView />)

    cardFor(container, 'Review parser behavior').click()
    expect(container.querySelector('.interact-actions')).toBe(null)

    cardFor(container, 'Discarded experiment').click()
    expect(container.querySelector('.interact-actions')).toBe(null)
  })
})
