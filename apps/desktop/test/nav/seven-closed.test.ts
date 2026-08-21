import { describe, expect, it } from 'vitest'
import { CATEGORIES, RAIL_ITEMS, VIEWS, categoryOf } from '../../src/nav'

describe('nav/seven-closed', () => {
  it('is exactly seven categories', () => {
    expect(CATEGORIES.length).toBe(7)
  })

  it('is frozen at runtime, not merely readonly in the types', () => {
    expect(Object.isFrozen(CATEGORIES)).toBe(true)
    expect(() => {
      ;(CATEGORIES as unknown as string[]).push('marketplace')
    }).toThrow()
    expect(CATEGORIES.length).toBe(7)
  })

  it('freezes the rail too, so an eighth item cannot appear that way either', () => {
    expect(Object.isFrozen(RAIL_ITEMS)).toBe(true)
    expect(() => {
      ;(RAIL_ITEMS as unknown as unknown[]).push({})
    }).toThrow()
    expect(RAIL_ITEMS.length).toBe(7)
  })

  it('freezes each rail item, so a category cannot be renamed in place', () => {
    expect(() => {
      ;(RAIL_ITEMS[0] as { label: string }).label = 'Mail'
    }).toThrow()
    expect(RAIL_ITEMS[0].label).toBe('Inbox')
  })

  it('assigns every view to one of the seven — a new surface joins one', () => {
    for (const view of VIEWS) {
      expect(CATEGORIES, view).toContain(categoryOf(view))
    }
  })

  it('leaves no category without a rail item, and no rail item without a category', () => {
    expect(RAIL_ITEMS.map((r) => r.category).sort()).toEqual([...CATEGORIES].sort())
  })
})
