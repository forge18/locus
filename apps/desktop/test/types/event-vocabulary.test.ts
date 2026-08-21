import { describe, expect, it } from 'vitest'
import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { EVENT_VERBS } from '../../src/types/event'
import type { AgentEvent, EventVerb } from '../../src/types/event'
import { SRC } from '../css'

/** The twelve, as PLAN.md §Canonical event vocabulary writes them. */
const CANONICAL = [
  'session_start',
  'user',
  'assistant',
  'thinking',
  'tool_call',
  'tool_result',
  'tool_error',
  'permission_request',
  'subagent_start',
  'subagent_stop',
  'aborted',
  'session_end',
]

describe('types/event-vocabulary', () => {
  it('is exactly the twelve verbs, in PLAN.md order', () => {
    expect([...EVENT_VERBS]).toEqual(CANONICAL)
  })

  it('typechecks, which is what makes the @ts-expect-error lines below assertions', () => {
    execFileSync('node_modules/.bin/tsc', ['--noEmit'], { cwd: resolve(SRC, '..'), stdio: 'pipe' })
  }, 60_000)

  it('rejects a thirteenth verb at the type level', () => {
    const ok: EventVerb = 'tool_error'
    expect(EVENT_VERBS).toContain(ok)
    // @ts-expect-error — there is no thirteenth verb
    const bad: EventVerb = 'tool_retry'
    void bad
  })

  it('matches the verbs PLAN.md itself lists', () => {
    const plan = readFileSync(resolve(SRC, '../../../PLAN.md'), 'utf8')
    const block = plan.match(
      /Canonical event vocabulary[\s\S]*?```\n([\s\S]*?)```/,
    )![1]
    const listed = block.split(/\s+/).filter(Boolean)
    expect(listed.sort()).toEqual([...CANONICAL].sort())
  })

  it('carries usage as nullable, so an unreported spend is unknown and not zero', () => {
    const unknown: AgentEvent = {
      id: 'e', runId: 'r', seq: 0, ts: '2026-08-20T00:00:00Z', verb: 'assistant',
      usage: null, raw: {},
    }
    expect(unknown.usage).toBeNull()
    // @ts-expect-error — usage is an object or null, never a bare number
    const wrong: AgentEvent = { ...unknown, usage: 0 }
    void wrong
  })

  it('keeps `raw` on every event, so a normalization bug is repairable by replay', () => {
    // @ts-expect-error — raw is required
    const noRaw: AgentEvent = { id: 'e', runId: 'r', seq: 0, ts: '', verb: 'user' }
    void noRaw
  })
})
