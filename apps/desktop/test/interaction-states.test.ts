import { describe, expect, it } from 'vitest'
import { read, rules } from './css'

const css = read('styles/interaction.css')
const rule = (sel: string) => rules(css).find((r) => r.selector === sel)

describe('interaction-states', () => {
  it('lifts one surface step on hover and one further on press', () => {
    expect(rule('.interactive')!.body).toContain('background: var(--surface-raised)')
    expect(rule('.interactive:hover')!.body).toContain('background: var(--surface-selected)')
    expect(rule('.interactive:active')!.body).toContain('background: var(--surface-elevated)')
  })

  it('gives a surface on the deep ground the same ladder, one step down', () => {
    expect(rule('.interactive-deep:hover')!.body).toContain('background: var(--surface-raised)')
    expect(rule('.interactive-deep:active')!.body).toContain('background: var(--surface-selected)')
  })

  it('draws focus as the accent outline with a 2px offset', () => {
    const focus = rule(':focus-visible')!
    expect(focus.body).toContain('outline: 2px solid var(--action-attention)')
    expect(focus.body).toContain('outline-offset: 2px')
  })

  it('expresses selection as an inset ring, never an outer glow', () => {
    const sel = rule('.selected')!
    expect(sel.body).toContain('background: var(--surface-selected)')
    expect(sel.body).toContain('var(--ring-sel)')
    const tokens = read('styles/tokens.css')
    expect(tokens).toContain('--ring-sel: inset 0 0 0 1px var(--action-attention)')
  })

  it('never animates a state change — the motion budget is two keyframes', () => {
    expect(css).not.toMatch(/transition:\s*(?!none)/)
  })
})
