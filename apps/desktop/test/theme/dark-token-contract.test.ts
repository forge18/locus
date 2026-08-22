import { describe, expect, it } from 'vitest'
import { read, rules } from '../css'

const tokens = read('styles/tokens.css')
const dark = rules(tokens).find((rule) => rule.selector === "[data-theme='dark']")

describe('theme/dark-token-contract', () => {
  it('defines the v2 Dark semantic palette under the dark theme selector', () => {
    expect(dark).toBeDefined()

    for (const token of [
      '--surface-ground: #1d2731',
      '--surface-chrome: #151d25',
      '--surface-raised: #22303c',
      '--surface-selected: #293947',
      '--text-primary: #eef2f6',
      '--action-attention: #ffbb39',
      '--status-working: #9184d9',
      '--status-success: #68ad91',
      '--status-danger: #df8a7d',
      '--data-1: #35495a',
      '--data-2: #4e6c81',
      '--data-3: #62869e',
      '--data-hi: #8fb8d6',
    ]) {
      expect(dark!.body).toContain(token)
    }
  })

  it('keeps v2 names as compatibility aliases to semantic roles', () => {
    for (const token of [
      '--bg: var(--surface-ground)',
      '--bg-deep: var(--surface-chrome)',
      '--sf: var(--surface-raised)',
      '--sf2: var(--surface-selected)',
      '--tx: var(--text-primary)',
      '--ac: var(--action-attention)',
      '--ac2: var(--status-working)',
      '--ok: var(--status-success)',
      '--bad: var(--status-danger)',
    ]) {
      expect(dark!.body).toContain(token)
    }
  })

  it('boots the document into the Dark value set before preferences load', () => {
    expect(read('../index.html')).toContain('<html lang="en" data-theme="dark">')
  })
})
