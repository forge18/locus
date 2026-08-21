import { expect, it } from 'vitest'
import { reachesTerminal, terminalOptions } from '../../src/panes/shell-config'

it('preserves Option-as-Meta and vim keystrokes in the terminal', () => {
  expect(terminalOptions.macOptionIsMeta).toBe(true)
  expect(reachesTerminal(new KeyboardEvent('keydown', { key: 'v', altKey: true }))).toBe(true)
})
