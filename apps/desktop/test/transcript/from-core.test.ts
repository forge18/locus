import { describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => {
  const invoke = vi.fn().mockResolvedValue(undefined)
  let latest: { onmessage: (event: unknown) => void } | undefined
  class Channel {
    onmessage: (event: unknown) => void = () => undefined

    constructor() {
      latest = this
    }
  }
  return { Channel, invoke, latest: () => latest }
})

vi.mock('@tauri-apps/api/core', () => ({ Channel: mocks.Channel, invoke: mocks.invoke }))

import { streamFromCore } from '../../src/transcript/from-core'

describe('transcript/from-core', () => {
  it('subscribes to the Rust Channel<Event> and forwards normalized payloads', async () => {
    const received: string[] = []
    await streamFromCore((event) => received.push(event.verb))

    expect(mocks.invoke).toHaveBeenCalledWith('telemetry_subscribe', { channel: mocks.latest() })
    mocks.latest()?.onmessage({
      id: 'event-1', runId: 'run-1', seq: 0, ts: '2026-01-01T00:00:00Z',
      verb: 'assistant', raw: {},
    })
    expect(received).toEqual(['assistant'])
  })
})
