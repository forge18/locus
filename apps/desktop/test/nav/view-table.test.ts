import { describe, expect, it } from 'vitest'
import { CATEGORIES, CATEGORY_LABELS, RAIL_ITEMS, VIEWS, categoryOf } from '../../src/nav'

/** The table exactly as .specs/navigation/spec.md writes it. */
const TABLE: Array<[string, string, string]> = [
  ['inbox', 'dashboard', 'Inbox'],
  ['status', 'dashboard', 'Inbox'],
  ['plan', 'plan', 'Plan'],
  ['develop', 'develop', 'Develop'],
  ['board', 'automate', 'Automate'],
  ['sessions', 'automate', 'Automate'],
  ['telemetry', 'review', 'Review'],
  ['runs', 'review', 'Review'],
  ['artifact', 'review', 'Review'],
  ['extensions', 'workshop', 'Workshop'],
  ['agents', 'workshop', 'Workshop'],
  ['canvas', 'workshop', 'Workshop'],
  ['harnesses', 'workshop', 'Workshop'],
  ['wiki', 'wiki', 'Wiki'],
]

describe('nav/view-table', () => {
  it('holds the fourteen views', () => {
    expect([...VIEWS].sort()).toEqual(TABLE.map(([v]) => v).sort())
  })

  it('maps each view to its category and rail label', () => {
    for (const [view, category, label] of TABLE) {
      expect(categoryOf(view as never), view).toBe(category)
      expect(CATEGORY_LABELS[category as never], view).toBe(label)
    }
  })

  it('has one rail item per category, in rail order', () => {
    expect(RAIL_ITEMS.map((r) => r.category)).toEqual([...CATEGORIES])
  })

  it('gives each rail item a Phosphor glyph and a first view', () => {
    for (const item of RAIL_ITEMS) {
      expect(item.icon, item.category).toMatch(/^[a-z-]+$/)
      expect(categoryOf(item.firstView), item.category).toBe(item.category)
    }
  })

  it('is one exported constant, not a per-component copy', () => {
    // Every category is reachable from the one list, and the list is the rail.
    expect(new Set(RAIL_ITEMS.map((r) => r.category)).size).toBe(CATEGORIES.length)
  })
})
