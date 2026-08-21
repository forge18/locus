import { describe, expect, it } from 'vitest'
import { VIEWS, format, parse, resolve } from '../../src/nav'
import type { View, ViewParams } from '../../src/nav'

/** One params set per view, covering the object form where the view has one. */
const CASES: Array<[View, ViewParams]> = [
  ['inbox', { project: 'tapestry' }],
  ['status', { project: 'tapestry' }],
  ['plan', { project: 'loom-db' }],
  ['develop', { project: 'weaver' }],
  ['telemetry', { project: 'texere' }],
  ['runs', { project: 'tapestry' }],
  ['runs', { project: 'weaver', sessionId: '5a71', runId: '9c02' }],
  ['extensions', { project: 'tapestry' }],
  ['harnesses', { project: 'tapestry' }],
  ['sessions', { project: 'tapestry' }],
  ['sessions', { project: 'tapestry', sessionId: '8f21' }],
  ['board', { project: 'loom-db' }],
  ['board', { project: 'loom-db', taskId: 't-004' }],
  ['artifact', { project: 'weaver' }],
  ['artifact', { project: 'weaver', artifactId: 'a-1' }],
  ['wiki', { project: 'texere' }],
  ['wiki', { project: 'texere', slug: 'event-vocabulary' }],
  ['canvas', { project: 'tapestry' }],
  ['canvas', { project: 'tapestry', workflowId: 'wf-1' }],
  ['canvas', { project: 'tapestry', workflowId: 'wf-1', executionId: 'ex-1' }],
  ['agents', { project: 'tapestry' }],
  ['agents', { project: 'tapestry', agentName: 'builder', agentVersion: '4' }],
]

describe('nav/locator-roundtrip', () => {
  it('resolves back to what it formatted, for every case', () => {
    for (const [view, params] of CASES) {
      const locator = format(view, params)
      expect(resolve(locator), locator).toEqual({ view, params })
    }
  })

  it('covers every one of the fourteen views', () => {
    expect(new Set(CASES.map(([v]) => v)).size).toBe(VIEWS.length)
  })

  it('formats back to the same string it parsed, for every case', () => {
    for (const [view, params] of CASES) {
      const locator = format(view, params)
      const back = resolve(locator)
      expect(format(back.view, back.params), locator).toBe(locator)
      expect(parse(locator).project).toBe(params.project)
    }
  })

  it('is stable under a second round trip — normalization converges', () => {
    for (const [view, params] of CASES) {
      const once = format(view, params)
      const twice = format(resolve(once).view, resolve(once).params)
      expect(twice).toBe(once)
    }
  })
})
