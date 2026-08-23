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
    expect(getByTestId('title-category').textContent).toBe('Workshop')
    expect(getByTestId('title-view').textContent).toBe('agents')
  })

  it('keeps Workshop lit on the rail — a drill-down is not a category', () => {
    const nav = createNavStore({ view: 'agents' })
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ))
    expect(getByTestId('project-rail')).toBeTruthy()
    expect(getByTestId('title-category').textContent).toBe('Workshop')
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
