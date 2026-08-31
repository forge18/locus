import { describe, expect, it } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { createNavStore } from '../../src/nav'
import { read, rules } from '../css'

const rule = (sel: string) => rules(read('screens/screens.css')).find((r) => r.selector === sel)
const mount = () => render(() => <InboxView nav={createNavStore()} />)

import { configureInboxStub } from "./inbox-stub";
configureInboxStub();

describe('inbox/layout', () => {
  it('is two panes', async () => {
    const { getByTestId } = mount()
    await waitFor(() => expect(getByTestId('inbox-detail')).toBeTruthy())
    expect(getByTestId('inbox-list')).toBeTruthy()
  })

  it('holds the left pane near 392px with a right hairline, and lets it flex', () => {
    const body = rule('.inbox-list')!.body
    expect(body).toContain('width: clamp(300px, 30%, 440px)')
    expect(body).toContain('flex: none')
    expect(body).toContain('border-right: 1px solid var(--border-subtle)')
  })

  it('lets the detail pane take the rest', () => {
    expect(rule('.inbox-detail')!.body).toContain('flex: 1')
  })

  it('scrolls the list, not the screen', () => {
    expect(rule('.inbox-list')!.body).toContain('overflow: auto')
  })

  it('opens on the first item, so the right pane is never blank on arrival', async () => {
    const { getByTestId } = mount()
    await waitFor(() =>
      expect(getByTestId('inbox-detail-title').textContent).toContain('sign-off'),
    )
  })
})
