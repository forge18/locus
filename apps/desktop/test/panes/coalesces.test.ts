import { expect, it } from 'vitest'
import { coalesce, type FrameScheduler } from '../../src/panes/coalesce'

it('panes/coalesces', () => {
  let callback: FrameRequestCallback | undefined
  const scheduler: FrameScheduler = { request: (next) => { callback = next; return 1 }, cancel: () => undefined }
  const renders: number[][] = []
  const stream = coalesce<number>((items) => renders.push([...items]), scheduler)
  stream.push(1); stream.push(2); stream.push(3)
  expect(renders).toEqual([])
  callback!(0)
  expect(renders).toEqual([[1, 2, 3]])
})

it('panes/coalesce-under-load', () => {
  let callback: FrameRequestCallback | undefined
  const scheduler: FrameScheduler = { request: (next) => { callback = next; return 1 }, cancel: () => undefined }
  let renders = 0
  const stream = coalesce<number>(() => { renders += 1 }, scheduler)
  for (let index = 0; index < 1_000; index += 1) stream.push(index)
  callback!(0)
  expect(renders).toBe(1)
})
