import { describe, expect, it } from 'vitest'
import { read, rules } from '../css'

const css = read('ui/ui.css')
const rule = (sel: string) => rules(css).find((r) => r.selector === sel)

describe('ui/button-outlined', () => {
  it('draws primary as an accent line, not an accent fill', () => {
    const primary = rule('.btn-primary')!
    expect(primary.body).toContain('color: var(--ac)')
    expect(primary.body).toContain('border-color: var(--ac)')
    // A background at all would make it a fill.
    expect(primary.body).not.toMatch(/background/)
  })

  it('leaves the base button transparent, so no variant inherits a fill', () => {
    expect(rule('.btn')!.body).toContain('background: transparent')
  })

  it('tints only on hover and press, and only as a wash', () => {
    for (const sel of ['.btn-primary:hover', '.btn-primary:active']) {
      const body = rule(sel)!.body
      expect(body).toMatch(/background: color-mix\(in srgb, var\(--ac\) \d+%, transparent\)/)
    }
  })

  it('never paints an accent background anywhere in the button rules', () => {
    for (const r of rules(css).filter((x) => x.selector.startsWith('.btn'))) {
      expect(r.body, `${r.selector}`).not.toMatch(/background:\s*var\(--ac\)/)
    }
  })
})
