import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { createNavStore } from '../../src/nav'
import { useInboxItems } from '../../src/data/inbox'
import { read, rules } from '../css'

const mount = () => render(() => <InboxView nav={createNavStore()} />)

describe('inbox/needs-you-header', () => {
  it('reads NEEDS YOU', () => {
    const { getByTestId } = mount()
    expect(getByTestId('needs-you-title').textContent).toBe('Needs you')
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.inbox-section-title',
    )!
    expect(rule.body).toContain('text-transform: uppercase')
  })

  it('sets the label in accent', () => {
    const rule = rules(read('screens/screens.css')).find(
      (r) => r.selector === '.inbox-section-title',
    )!
    expect(rule.body).toContain('color: var(--action-attention)')
  })

  it('counts the items that are actually there', () => {
    const { getByTestId } = mount()
    expect(getByTestId('needs-you-note').textContent).toContain(`${useInboxItems().length} items`)
  })

  it('carries the note that silence is the default', () => {
    const { getByTestId } = mount()
    expect(getByTestId('needs-you-note').textContent).toContain('silence is the default')
  })

  it('says "item" rather than "items" for one', () => {
    const { getByTestId } = mount()
    const [first] = useInboxItems()
    getByTestId(`inbox-card-${first.id}`).click()
    getByTestId('inbox-approve').click()
    getByTestId('inbox-approve').click()
    expect(getByTestId('needs-you-note').textContent).toContain('1 item ·')
  })
})
