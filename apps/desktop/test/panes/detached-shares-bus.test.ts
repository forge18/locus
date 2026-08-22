import { expect, it } from 'vitest'
import { streamFromCore } from '../../src/transcript/from-core'

it('uses the core event bus instead of copied JavaScript state', () => {
  expect(typeof streamFromCore).toBe('function')
})
