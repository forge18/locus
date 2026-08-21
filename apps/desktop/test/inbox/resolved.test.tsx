import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { createNavStore } from '../../src/nav'
import { useResolvedToday } from '../../src/data/inbox'
import { read, rules } from '../css'

const mount = () => render(() => <InboxView nav={createNavStore()} />)

describe('inbox/resolved', () => {
  it('is headed RESOLVED TODAY', () => {
    const { getByTestId } = mount()
    expect(getByTestId('resolved-title').textContent).toBe('Resolved today')
  })

  it('lists one row per resolved item', () => {
    const { getByTestId } = mount()
    expect(getByTestId('inbox-resolved').querySelectorAll('.inbox-resolved-row').length).toBe(
      useResolvedToday().length,
    )
  })

  it('dims the rows to .6 — done is context, not work', () => {
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.inbox-resolved-row',
    )!
    expect(rule.body).toMatch(/opacity:\s*\.6/)
  })

  it('shows an icon, the title and the age on each row', () => {
    const { getByTestId } = mount()
    const row = getByTestId(`resolved-${useResolvedToday()[0].id}`)
    expect(row.querySelector('use')).not.toBe(null)
    expect(row.textContent).toContain(useResolvedToday()[0].title)
    expect(row.textContent).toContain('1h')
  })

  it('sits below the live items, not among them', () => {
    const { getByTestId } = mount()
    const list = getByTestId('inbox-list')
    const cards = list.querySelectorAll('.inbox-card')
    const resolved = getByTestId('inbox-resolved')
    expect(
      cards[cards.length - 1].compareDocumentPosition(resolved) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })
})
