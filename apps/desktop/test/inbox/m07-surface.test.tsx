import { render } from '@solidjs/testing-library'
import { describe, expect, it } from 'vitest'
import { InboxView } from '../../src/screens/inbox/InboxView'
import { useInboxItems } from '../../src/data/inbox'
import { parse } from '../../src/nav'
import type { NavStore } from '../../src/nav'

const nav = { open: () => ({}) } as unknown as NavStore
const mount = () => render(() => <InboxView nav={nav} />)

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe('inbox/m07-surface', () => {
  it('renders the To do and Completed tabs, throughput, and a live item log', () => {
    const { getByTestId } = mount()

    expect(getByTestId('inbox-tab-todo').textContent).toContain('To do')
    expect(getByTestId('inbox-tab-completed').textContent).toContain('Completed')
    expect(getByTestId('inbox-throughput').textContent).toContain('3 / 6 per hour')
    expect(getByTestId('inbox-project-filter-note').textContent).toContain('Filters this list only')
    expect(getByTestId('inbox-items').getAttribute('aria-live')).toBe('polite')
    expect(getByTestId('inbox-items').getAttribute('role')).toBe('log')
    expect(getByTestId('inbox-why').textContent).toContain('blocked, not idle')
    expect(getByTestId('inbox-cost').textContent).toContain('No tokens burn while blocked.')
  })

  it('groups completed rows by day and shows their resolution time', () => {
    const { getByTestId } = mount()
    getByTestId('inbox-tab-completed').click()

    const groups = getByTestId('inbox-completed-items').querySelectorAll('.inbox-completed-day')
    expect(groups.length).toBeGreaterThan(1)
    expect(getByTestId('resolved-time-rs-1').textContent).toContain('18m')
    expect(getByTestId('resolved-time-rs-3').textContent).toContain('1h 22m')
  })

  it('uses canonical /view locators for each inbox destination', () => {
    for (const item of useInboxItems()) {
      expect(item.opensAt).toMatch(/^locus:\/\/[^/]+\/view\//)
      expect(() => parse(item.opensAt), item.id).not.toThrow()
    }
  })
})
