import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Shell } from '../../src/shell/Shell'
import { RAIL_ITEMS, createNavStore } from '../../src/nav'

describe('nav/rail-first-view', () => {
  it('lands each rail click on the category first view', () => {
    const nav = createNavStore()
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    for (const item of RAIL_ITEMS) {
      getByTestId(`rail-${item.category}`).click()
      expect(nav.view(), item.category).toBe(item.firstView)
    }
  })

  it('lands on the documented first view, not on what was last open there', () => {
    const nav = createNavStore()
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    getByTestId('rail-review').click()
    nav.go('runs') // drill within Review
    expect(nav.view()).toBe('runs')

    getByTestId('rail-workshop').click()
    getByTestId('rail-review').click()
    // Back to Telemetry, not to the Runs tab that was open a moment ago.
    expect(nav.view()).toBe('telemetry')
  })

  it('lights the category it landed on', () => {
    const nav = createNavStore()
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    getByTestId('rail-automate').click()
    expect(getByTestId('rail-automate').getAttribute('aria-current')).toBe('true')
    expect(nav.view()).toBe('board')
  })
})
