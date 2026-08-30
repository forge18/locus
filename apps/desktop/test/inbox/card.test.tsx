import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxCard } from '../../src/screens/inbox/InboxCard'
import { PENDING } from './deliveries'

const ITEM = PENDING[0]
const mount = (item = ITEM, selected = false) =>
  render(() => <InboxCard item={item} selected={selected} onSelect={() => {}} />)

describe('inbox/card', () => {
  it('shows the subject', () => {
    const { getByTestId } = mount()
    expect(getByTestId(`inbox-card-${ITEM.id}`).textContent).toContain(ITEM.subject)
  })

  it('right-aligns the age', () => {
    const { getByTestId } = mount()
    expect(getByTestId('inbox-card-age').textContent).toBe('26m')
    expect(getByTestId('inbox-card-age').className).toContain('inbox-card-age')
  })

  it('rolls the age over to hours rather than showing three digits of minutes', () => {
    const { getByTestId } = mount(PENDING[1])
    expect(getByTestId('inbox-card-age').textContent).toBe('2h')
  })

  it('carries the project · sender subline, with the sender in mono', () => {
    const { getByTestId } = mount()
    const sub = getByTestId('inbox-card-sub')
    expect(sub.textContent).toContain('tapestry')
    expect(sub.querySelector('.mono')!.textContent).toBe('agent')
  })

  it('selects when it is clicked', () => {
    let selected = 0
    const { getByTestId } = render(() => (
      <InboxCard item={ITEM} selected={false} onSelect={() => selected++} />
    ))
    getByTestId(`inbox-card-${ITEM.id}`).click()
    expect(selected).toBe(1)
  })
})
