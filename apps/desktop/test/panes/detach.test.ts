import { expect, it } from 'vitest'
import { detachedMode } from '../../src/panes/detach'

it('panes/detach-window', () => {
  history.replaceState({}, '', '/?detached=true')
  expect(detachedMode()).toBe(true)
})
it('panes/detached-shares-bus', async () => {
  const transcript = await import('../../src/transcript/from-core')
  expect(typeof transcript.streamFromCore).toBe('function')
})
