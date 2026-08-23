import { describe, expect, it } from 'vitest'
import { read, rules } from '../css'

const tokens = read('styles/tokens.css')
const screenRules = rules(read('screens/screens.css'))
const rule = (selector: string) => screenRules.find((candidate) => candidate.selector === selector)

describe('design/desktop-accent-roles', () => {
  it('defines separate attention, working, and magnitude roles', () => {
    for (const token of ['--ac: var(--action-attention)', '--ac2: var(--status-working)']) {
      expect(tokens).toContain(token)
    }

    for (const token of ['--data-1:', '--data-2:', '--data-3:', '--data-hi:']) {
      expect(tokens).toContain(token)
    }
  })

  it('uses the data ramp rather than attention for magnitude bars', () => {
    expect(rule('.sparkline-bar')?.body).toContain('background: var(--data-2)')
    expect(rule('.bar-fill')?.body).toContain('background: var(--data-3)')

    for (const selector of ['.sparkline-bar', '.bar-fill']) {
      expect(rule(selector)?.body).not.toContain('var(--ac)')
      expect(rule(selector)?.body).not.toContain('var(--ac2)')
    }
  })
})
