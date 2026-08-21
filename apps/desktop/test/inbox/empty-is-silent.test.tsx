import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { createNavStore } from '../../src/nav'
import { useInboxItems } from '../../src/data/inbox'
import { read } from '../css'

/** Resolve everything, which is the only way to reach empty from the fixtures. */
const emptied = () => {
  const r = render(() => <InboxView nav={createNavStore()} />)
  for (let i = 0; i < useInboxItems().length; i++) r.getByTestId('inbox-approve').click()
  return r
}

describe('inbox/empty-is-silent', () => {
  it('says "Nothing needs you"', () => {
    const { container } = emptied()
    expect(container.textContent).toContain('Nothing needs you')
  })

  it('shows no cards at all', () => {
    const { container } = emptied()
    expect(container.querySelectorAll('.inbox-card').length).toBe(0)
  })

  it('shows no spinner — silence is the default here, not loading', () => {
    const { container } = emptied()
    expect(container.querySelector('.pulse')).toBe(null)
    expect(container.querySelector('.skeleton-rows')).toBe(null)
    expect(container.textContent?.toLowerCase()).not.toContain('loading')
  })

  it('states a reason rather than "No items"', () => {
    const { container } = emptied()
    expect(container.textContent).not.toContain('No items')
    expect(container.querySelector('[data-testid="empty-pane"]')).not.toBe(null)
  })

  it('counts zero in the header without breaking the sentence', () => {
    const { getByTestId } = emptied()
    expect(getByTestId('needs-you-note').textContent).toBe('0 items · silence is the default')
  })

  it('never puts a notification here — the screen has no such path', () => {
    // An item that only reports something happened is not inbox work, and the
    // component takes no shape that could carry one.
    expect(read('screens/inbox/InboxView.tsx')).not.toMatch(/notification|toast|notify/i)
  })
})
