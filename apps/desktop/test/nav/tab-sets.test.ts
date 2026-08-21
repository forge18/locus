import { describe, expect, it } from 'vitest'
import { CATEGORIES, activeTabFor, tabsFor } from '../../src/nav'

describe('nav/tab-sets', () => {
  it('gives dashboard Inbox then Status', () => {
    expect(tabsFor('dashboard').map((t) => t.label)).toEqual(['Inbox', 'Status'])
  })

  it('gives Automate Kanban then Agents, in that order', () => {
    const tabs = tabsFor('automate')
    expect(tabs.map((t) => t.label)).toEqual(['Kanban', 'Agents'])
    expect(tabs.map((t) => t.view)).toEqual(['board', 'sessions'])
  })

  it('gives Review its three', () => {
    expect(tabsFor('review').map((t) => t.view)).toEqual(['telemetry', 'runs', 'artifact'])
  })

  it('gives Workshop Extensions, Workflow, Harnesses', () => {
    expect(tabsFor('workshop').map((t) => t.label)).toEqual([
      'Extensions',
      'Workflow',
      'Harnesses',
    ])
  })

  it('gives Plan, Develop and Wiki none', () => {
    for (const category of ['plan', 'develop', 'wiki'] as const) {
      expect(tabsFor(category), category).toEqual([])
    }
  })

  it('covers every category, so no category is missing a tab set', () => {
    for (const c of CATEGORIES) expect(Array.isArray(tabsFor(c)), c).toBe(true)
  })

  it('lights the tab for a view that has one, and Extensions for the drill-down', () => {
    expect(activeTabFor('runs')).toBe('runs')
    expect(activeTabFor('agents')).toBe('extensions')
    expect(activeTabFor('plan')).toBe(null)
  })
})
