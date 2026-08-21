import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Shell } from '../../src/shell/Shell'
import { createNavStore } from '../../src/nav'
import { read, rules, stripComments } from '../css'

describe('nav/tab-click', () => {
  it('navigates to the view the tab names', () => {
    const nav = createNavStore({ view: 'telemetry' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    getByTestId('tab-runs').click()
    expect(nav.view()).toBe('runs')
    getByTestId('tab-artifact').click()
    expect(nav.view()).toBe('artifact')
  })

  it('stays inside the category, so the rail does not move', () => {
    const nav = createNavStore({ view: 'telemetry' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    getByTestId('tab-runs').click()
    expect(getByTestId('rail-review').getAttribute('aria-current')).toBe('true')
  })

  it('is instant — the tab and its bar carry no transition', () => {
    const shell = stripComments(read('shell/shell.css'))
    const ui = rules(read('ui/ui.css'))
    expect(shell).not.toMatch(/transition/)
    for (const r of ui.filter((x) => x.selector.startsWith('.tab'))) {
      expect(r.body, r.selector).not.toMatch(/transition|animation/)
    }
  })

  it('marks the new tab selected and the old one not', () => {
    const nav = createNavStore({ view: 'telemetry' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    expect(getByTestId('tab-telemetry').getAttribute('data-selected')).toBe('')
    getByTestId('tab-runs').click()
    expect(getByTestId('tab-telemetry').getAttribute('data-selected')).toBe(null)
    expect(getByTestId('tab-runs').getAttribute('data-selected')).toBe('')
  })
})
