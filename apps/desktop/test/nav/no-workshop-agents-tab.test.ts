import { describe, expect, it } from 'vitest'
import { CATEGORIES, tabsFor } from '../../src/nav'

describe('nav/no-workshop-agents-tab', () => {
  it('has no `agents` tab in the Workshop set', () => {
    expect(tabsFor('workshop').map((t) => t.view)).not.toContain('agents')
  })

  it('has no tab labelled Agents anywhere in Workshop', () => {
    expect(tabsFor('workshop').map((t) => t.label)).not.toContain('Agents')
  })

  it("keeps Automate's Agents tab, which is the live session list and a different thing", () => {
    const automate = tabsFor('automate').find((t) => t.label === 'Agents')!
    expect(automate.view).toBe('sessions')
  })

  it('has `agents` appear in no category tab set at all', () => {
    for (const c of CATEGORIES) {
      expect(tabsFor(c).map((t) => t.view), c).not.toContain('agents')
    }
  })
})
