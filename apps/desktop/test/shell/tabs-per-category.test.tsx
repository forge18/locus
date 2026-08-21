import { describe, expect, it } from 'vitest'
import { render } from '@solidjs/testing-library'
import { TabBar } from '../../src/shell/TabBar'
import type { View } from '../../src/nav'

const tabsFor = (view: View) => {
  const { getByTestId, unmount } = render(() => (
    <TabBar view={view} onNavigate={() => {}} locator="x" />
  ))
  const labels = [...getByTestId('tabbar-tabs').querySelectorAll('.tab')].map((t) => t.textContent)
  unmount()
  return labels
}

describe('shell/tabs-per-category', () => {
  it('shows the dashboard tabs in order', () => {
    expect(tabsFor('inbox')).toEqual(['Inbox', 'Status'])
  })

  it('shows Kanban before Agents on Automate', () => {
    expect(tabsFor('board')).toEqual(['Kanban', 'Agents'])
  })

  it('shows the three Review tabs', () => {
    expect(tabsFor('telemetry')).toEqual(['Telemetry', 'Runs', 'Artifacts'])
  })

  it('shows the three Workshop tabs, and no Agents tab among them', () => {
    const tabs = tabsFor('extensions')
    expect(tabs).toEqual(['Extensions', 'Workflow', 'Harnesses'])
    expect(tabs).not.toContain('Agents')
  })

  it('shows none for Plan, Develop and Wiki', () => {
    expect(tabsFor('plan')).toEqual([])
    expect(tabsFor('develop')).toEqual([])
    expect(tabsFor('wiki')).toEqual([])
  })

  it('keeps Extensions lit while agent definitions are open', () => {
    const { getByTestId } = render(() => (
      <TabBar view="agents" onNavigate={() => {}} locator="tapestry/agent/builder@4" />
    ))
    const selected = getByTestId('tabbar-tabs').querySelectorAll('.tab[data-selected]')
    expect(selected.length).toBe(1)
    expect(selected[0].textContent).toBe('Extensions')
  })

  it('navigates to the tab that was clicked', () => {
    const landed: string[] = []
    const { getByTestId } = render(() => (
      <TabBar view="telemetry" onNavigate={(v) => landed.push(v)} locator="x" />
    ))
    getByTestId('tab-runs').click()
    getByTestId('tab-artifact').click()
    expect(landed).toEqual(['runs', 'artifact'])
  })
})
