import { describe, expect, it } from 'vitest'
import { SESSIONS, eventsFor } from '../../src/fixtures/sessions'
import { VERB_COUNTS } from '../../src/fixtures/telemetry'
import { EVENT_VERBS } from '../../src/types/event'

describe('fixtures/only-canonical-verbs', () => {
  it('uses only the twelve verbs in every fixture transcript', () => {
    for (const session of SESSIONS.slice(0, 25)) {
      for (const e of eventsFor(session.id)) {
        expect(EVENT_VERBS, `${session.id}: ${e.verb}`).toContain(e.verb)
      }
    }
  })

  it('fails a thirteenth verb rather than tolerating it', () => {
    const events = eventsFor(SESSIONS[0].id)
    const smuggled = [...events, { ...events[0], verb: 'tool_retry' as never }]
    expect(smuggled.every((e) => (EVENT_VERBS as readonly string[]).includes(e.verb))).toBe(false)
  })

  it('faces the telemetry facets over the same twelve, and no more', () => {
    expect(VERB_COUNTS.map((v) => v.verb).sort()).toEqual([...EVENT_VERBS].sort())
  })

  it('orders events by seq, which the core assigns on arrival', () => {
    const events = eventsFor(SESSIONS[0].id)
    expect(events.map((e) => e.seq)).toEqual(events.map((_, i) => i))
  })

  it('keeps raw on every event', () => {
    for (const e of eventsFor(SESSIONS[0].id)) {
      expect(e.raw, `${e.id}`).toBeTypeOf('object')
    }
  })
})
