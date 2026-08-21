import { describe, expect, it } from 'vitest'
import { allSource, declarations, read, rules, stripComments } from './css'

describe('no-default-focus-ring', () => {
  const interaction = read('styles/interaction.css')

  it('sets the accent outline on a universal :focus-visible, so nothing is missed', () => {
    const r = rules(interaction).find((x) => x.selector === ':focus-visible')
    expect(r, 'no universal :focus-visible rule').toBeDefined()
    expect(r!.body).toContain('outline: 2px solid var(--ac)')
  })

  it('removes an outline nowhere without putting the accent one back', () => {
    for (const [file, contents] of allSource()) {
      const body = stripComments(contents)
      for (const value of declarations(body, 'outline')) {
        expect(value, `${file}: outline: ${value}`).toMatch(/2px solid var\(--ac\)/)
      }
      for (const value of declarations(body, 'outline-width')) {
        expect(value, `${file}: outline-width: ${value}`).not.toMatch(/^0/)
      }
    }
  })

  it('does not reach for :focus, which would ring a mouse press too', () => {
    for (const [file, contents] of allSource()) {
      const selectors = rules(contents).map((r) => r.selector)
      for (const sel of selectors) {
        expect(sel, `${file}: ${sel}`).not.toMatch(/:focus(?![-\w])/)
      }
    }
  })
})
