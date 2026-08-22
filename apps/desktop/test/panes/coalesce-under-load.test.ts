import { expect, it } from 'vitest'
import { coalesce, type FrameScheduler } from '../../src/panes/coalesce'

it('keeps 1,000 sends in one frame render', () => {
  let callback: FrameRequestCallback | undefined
  const scheduler: FrameScheduler = {
    request: (next) => { callback = next; return 1 },
    cancel: () => undefined,
  }
  let renders = 0
  const stream = coalesce<number>(() => { renders += 1 }, scheduler)

  for (let index = 0; index < 1_000; index += 1) stream.push(index)
  callback!(0)

  expect(renders).toBe(1)
})
