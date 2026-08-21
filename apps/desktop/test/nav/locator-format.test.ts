import { describe, expect, it } from 'vitest'
import { format } from '../../src/nav'

describe('nav/locator-format', () => {
  it('formats a view that addresses no object', () => {
    expect(format('inbox', { project: 'tapestry' })).toBe('locus://tapestry/inbox')
    expect(format('telemetry', { project: 'weaver' })).toBe('locus://weaver/telemetry')
  })

  it('formats each object kind from its view and params', () => {
    expect(format('sessions', { project: 'tapestry', sessionId: '8f21' })).toBe(
      'locus://tapestry/session/8f21',
    )
    expect(format('board', { project: 'loom-db', taskId: 't-004' })).toBe(
      'locus://loom-db/task/t-004',
    )
    expect(format('artifact', { project: 'weaver', artifactId: 'a-1' })).toBe(
      'locus://weaver/artifact/a-1',
    )
    expect(format('wiki', { project: 'texere', slug: 'event-vocabulary' })).toBe(
      'locus://texere/page/event-vocabulary',
    )
    expect(format('canvas', { project: 'tapestry', workflowId: 'wf-1' })).toBe(
      'locus://tapestry/workflow/wf-1',
    )
  })

  it('formats an agent back to name@version', () => {
    expect(format('agents', { project: 'tapestry', agentName: 'builder', agentVersion: '4' })).toBe(
      'locus://tapestry/agent/builder@4',
    )
  })

  it('appends the sub-object where the view carries one', () => {
    expect(format('runs', { project: 'tapestry', sessionId: '8f21', runId: '3c04' })).toBe(
      'locus://tapestry/session/8f21/run/3c04',
    )
    expect(
      format('canvas', { project: 'tapestry', workflowId: 'wf-1', executionId: 'ex-1' }),
    ).toBe('locus://tapestry/workflow/wf-1/execution/ex-1')
  })

  it('drops a run id from the sessions view — a finished run is Review\'s object', () => {
    // A live session is operated in Automate; the run form addresses Review.
    expect(format('sessions', { project: 'tapestry', sessionId: '8f21', runId: '3c04' })).toBe(
      'locus://tapestry/session/8f21',
    )
  })

  it('falls back to the runs screen when the run half is missing', () => {
    expect(format('runs', { project: 'tapestry', sessionId: '8f21' })).toBe(
      'locus://tapestry/runs',
    )
  })

  it('falls back to the view form when the object id is absent', () => {
    // The screen, rather than one object on it.
    expect(format('sessions', { project: 'tapestry' })).toBe('locus://tapestry/sessions')
    expect(format('board', { project: 'tapestry' })).toBe('locus://tapestry/board')
  })
})
