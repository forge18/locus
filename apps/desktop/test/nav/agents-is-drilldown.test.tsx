import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { Shell } from '../../src/shell/Shell'
import { BackLink } from '../../src/nav/BackLink'
import { createNavStore, drilldownParent, tabsFor } from '../../src/nav'

describe('nav/agents-is-drilldown', () => {
  it('names Extensions as the view agent definitions were entered from', () => {
    expect(drilldownParent('agents')).toBe('extensions')
    expect(drilldownParent('extensions')).toBe(null)
    expect(drilldownParent('sessions')).toBe(null)
  })

  it('keeps the Extensions tab lit while agent definitions are open', () => {
    const nav = createNavStore({ view: 'agents' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    const selected = getByTestId('tabbar-tabs').querySelectorAll('.tab[data-selected]')
    expect(selected.length).toBe(1)
    expect(selected[0].textContent).toBe('Extensions')
  })

  it('keeps Workshop lit on the rail — a drill-down is not a category', () => {
    const nav = createNavStore({ view: 'agents' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    expect(getByTestId('rail-workshop').getAttribute('aria-current')).toBe('true')
    expect(getByTestId('rail').querySelectorAll('.rail-item').length).toBe(7)
  })

  it('renders the back link, labelled with the tab it returns to', () => {
    const nav = createNavStore({ view: 'agents' })
    const { getByTestId } = render(() => <BackLink nav={nav} />)
    expect(getByTestId('drilldown-back').textContent).toBe('Extensions')
  })

  it('renders no back link anywhere that is not a drill-down', () => {
    const nav = createNavStore({ view: 'extensions' })
    const { queryByTestId } = render(() => <BackLink nav={nav} />)
    expect(queryByTestId('drilldown-back')).toBe(null)
  })

  it('goes back to Extensions when the link is followed', () => {
    const nav = createNavStore({ view: 'agents' })
    const { getByTestId } = render(() => <BackLink nav={nav} />)
    getByTestId('drilldown-back').click()
    expect(nav.view()).toBe('extensions')
  })

  it('is not a Workshop tab, so it cannot be reached as one', () => {
    expect(tabsFor('workshop').map((t) => t.view)).not.toContain('agents')
  })
})
