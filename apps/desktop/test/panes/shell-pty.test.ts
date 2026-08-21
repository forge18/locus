import { expect, it } from 'vitest'
import { terminalOptions } from '../../src/panes/shell-config'

it('configures xterm with a PTY-safe Option modifier', () => {
  expect(terminalOptions).toMatchObject({ macOptionIsMeta: true, convertEol: true })
})
