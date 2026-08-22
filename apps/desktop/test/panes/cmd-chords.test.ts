import { expect, it } from 'vitest'
import { reachesTerminal } from '../../src/panes/shell-config'

it('leaves Cmd chords for Rust-registered application accelerators', () => {
  expect(reachesTerminal(new KeyboardEvent('keydown', { key: 'k', metaKey: true }))).toBe(false)
})
