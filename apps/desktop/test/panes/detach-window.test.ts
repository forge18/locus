import { expect, it } from 'vitest'
import { detachedMode } from '../../src/panes/detach'

it('opens the same app in detached mode in a new window', () => {
  history.replaceState({}, '', '/?detached=true')
  expect(detachedMode()).toBe(true)
})
