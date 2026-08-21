import { describe, expect, it } from 'vitest'
import { KINDS, parse } from '../../src/nav'

describe('nav/locator-parse', () => {
  it('parses a session', () => {
    expect(parse('locus://tapestry/session/8f21')).toEqual({
      project: 'tapestry', kind: 'session', id: '8f21', subId: null,
    })
  })

  it('parses a session with its run', () => {
    expect(parse('locus://tapestry/session/8f21/run/3c04')).toEqual({
      project: 'tapestry', kind: 'session', id: '8f21', subId: '3c04',
    })
  })

  it('parses a task, an artifact and a page', () => {
    expect(parse('locus://loom-db/task/t-004').kind).toBe('task')
    expect(parse('locus://weaver/artifact/a-1').id).toBe('a-1')
    expect(parse('locus://texere/page/notification-sinks').id).toBe('notification-sinks')
  })

  it('parses a workflow, with and without an execution', () => {
    expect(parse('locus://tapestry/workflow/wf-1').subId).toBe(null)
    expect(parse('locus://tapestry/workflow/wf-1/execution/ex-1').subId).toBe('ex-1')
  })

  it('parses an agent as name@version', () => {
    expect(parse('locus://tapestry/agent/builder@4')).toEqual({
      project: 'tapestry', kind: 'agent', id: 'builder@4', subId: null,
    })
  })

  it('covers all six kinds', () => {
    expect([...KINDS]).toEqual(['session', 'task', 'artifact', 'page', 'workflow', 'agent'])
  })

  it('parses the view form, which addresses a screen rather than an object', () => {
    expect(parse('locus://tapestry/inbox')).toEqual({
      project: 'tapestry', kind: null, id: 'inbox', subId: null,
    })
  })

  it('keeps the project as a path segment, which is what makes it a filter', () => {
    for (const p of ['tapestry', 'loom-db', 'weaver', 'texere']) {
      expect(parse(`locus://${p}/inbox`).project).toBe(p)
    }
  })
})
