import { describe, expect, it } from 'vitest'
import { read, rules, type Rule } from '../css'

// Stacking is a ladder, not a pile of ad-hoc numbers: ui.css documents the
// tiers and screens take one. Reading the CSS is the honest way to pin the
// contract — jsdom does not implement the cascade.
const ui = rules(read('ui/ui.css'))
const projects = rules(read('screens/projects/projects.css'))
const dispatch = rules(read('screens/dispatch/dispatch.css'))

const tier = (sheet: Rule[], selector: string): number => {
  const rule = sheet.find((candidate) => candidate.selector === selector)
  expect(rule, `${selector} exists`).toBeDefined()
  const declared = rule!.body.match(/(?:^|;)\s*z-index:\s*(\d+)/)
  expect(declared, `${selector} declares a z-index`).not.toBeNull()
  return Number(declared![1])
}

describe('ui/z-ladder', () => {
  it('keeps the chrome ladder ordered: scrim < sheet < popups < toasts', () => {
    expect(tier(ui, '.overlay')).toBe(50)
    expect(tier(ui, '.sheet')).toBe(51)
    expect(tier(ui, '.tooltip')).toBe(60)
    expect(tier(ui, '.menu')).toBe(60)
    expect(tier(ui, '.toast-region')).toBe(70)
  })

  it('puts the projects editor overlay on the modal tier', () => {
    const editor = tier(projects, '.project-editor-overlay')
    expect(editor).toBe(80)
    expect(editor).toBeGreaterThan(tier(ui, '.toast-region'))
    expect(editor).toBeGreaterThan(tier(ui, '.menu'))
  })

  it('puts the dispatch stop-everything backdrop on the modal tier', () => {
    const stop = tier(dispatch, '.dispatch-dialog-backdrop')
    expect(stop).toBe(80)
    expect(stop).toBeGreaterThan(tier(ui, '.toast-region'))
    expect(stop).toBeGreaterThan(tier(ui, '.menu'))
  })
})
