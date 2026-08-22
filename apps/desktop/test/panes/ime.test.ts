import { expect, it } from 'vitest'
import { reachesTerminal } from '../../src/panes/shell-config'

it('passes IME composition and dead-key events through to xterm', () => {
  expect(reachesTerminal(new KeyboardEvent('compositionstart', { key: 'Dead' }))).toBe(true)
  expect(reachesTerminal(new KeyboardEvent('keydown', { key: 'Dead' }))).toBe(true)
})
