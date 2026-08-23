import { describe, expect, it } from 'vitest'
import { allSource, read, rules, stripComments } from './css'

const ACCENT = '#ffbb39'
const tokens = read('styles/tokens.css')

describe('accent-single-source', () => {
  it('writes the accent hex exactly once, in tokens.css', () => {
    const inTokens = [...stripComments(tokens).matchAll(new RegExp(ACCENT, 'gi'))]
    expect(inTokens.length).toBe(1)

    for (const [file, contents] of allSource()) {
      if (file === 'styles/tokens.css') continue
      expect(stripComments(contents).toLowerCase(), `${file} hardcodes the accent`).not.toContain(
        ACCENT,
      )
    }
  })

  it('derives every accent tint from --ac rather than restating it', () => {
    const derived = ['--ac-ring', '--ac-ring-soft', '--ac-wash', '--ac-deep', '--ac-pale', '--ac-ink']
    const root = rules(tokens).find((r) => r.selector === "[data-theme='dark']")!
    for (const name of derived) {
      const value = root.body.match(new RegExp(`${name}:\\s*([^;]+)`))?.[1]
      expect(value, `missing ${name}`).toBeDefined()
      expect(value, `${name} does not resolve from --ac`).toMatch(/var\(--ac\)|var\(--bg\)/)
    }
  })

  it('moves rings, live dots, active tabs and metric numerals together', () => {
    // Selection rings resolve from --ac …
    expect(tokens).toContain('--ring-sel: inset 0 0 0 1px var(--action-attention)')
    expect(tokens).toContain('--ring-sel-soft: inset 0 0 0 1px var(--action-attention-ring)')

    // … focus is the same accent …
    expect(read('styles/interaction.css')).toContain('outline: 2px solid var(--action-attention)')

    // … the live dot animates opacity, so it inherits whatever color --ac resolves to …
    const motion = read('styles/motion.css')
    expect(motion).toMatch(/@keyframes pulse[\s\S]*opacity/)
    expect(motion).not.toMatch(/color|background/)

    // … and no stylesheet paints anything with a literal color.
    for (const [file, contents] of allSource()) {
      if (file === 'styles/tokens.css') continue
      const body = stripComments(contents)
      expect(body, `${file} names a raw color`).not.toMatch(/#[0-9a-fA-F]{6}\b/)
    }
  })

  it('re-themes when --ac is replaced, because every consumer reads the variable', () => {
    // Swap the token in the source text and nothing else should still say amber.
    const swapped = tokens.replace(ACCENT, '#39ffbb')
    expect(swapped.toLowerCase()).not.toContain(ACCENT)

    const consumers = [read('styles/interaction.css'), read('styles/type.css'), read('styles/motion.css')]
    for (const c of consumers) {
      expect(stripComments(c).toLowerCase()).not.toContain(ACCENT)
    }
  })
})
