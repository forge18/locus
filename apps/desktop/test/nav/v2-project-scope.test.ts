import { describe, expect, it } from 'vitest'
import { format, formatV2Locator, resolve, resolveV2Locator } from '../../src/nav'

describe('nav/v2-project-scope', () => {
  it('formats global routes with an explicit global scope', () => {
    expect(formatV2Locator('inbox')).toBe('locus://global/inbox')
    expect(resolveV2Locator('locus://global/dispatch-runs')).toEqual({
      route: 'dispatch-runs',
      scope: { kind: 'global' },
    })
  })

  it('formats project routes with an explicit project scope and project segment', () => {
    expect(formatV2Locator('plan-conversation', 'tapestry')).toBe(
      'locus://project/tapestry/plan-conversation',
    )
    expect(resolveV2Locator('locus://project/loom-db/review-telemetry')).toEqual({
      route: 'review-telemetry',
      scope: { kind: 'project', project: 'loom-db' },
    })
  })

  it('rejects routes addressed with the wrong scope or an implicit v1 scope', () => {
    expect(() => formatV2Locator('inbox', 'tapestry')).toThrow(/scope:/)
    expect(() => formatV2Locator('plan-conversation')).toThrow(/project:/)
    expect(() => resolveV2Locator('locus://global/plan-conversation')).toThrow(/scope:/)
    expect(() => resolveV2Locator('locus://project/tapestry/inbox')).toThrow(/scope:/)
    expect(() => resolveV2Locator('locus://tapestry/inbox')).toThrow(/scope:/)
  })

  it('preserves the v1 fixture resolver during the migration', () => {
    expect(format('inbox', { project: 'tapestry' })).toBe('locus://tapestry/inbox')
    expect(resolve('locus://tapestry/inbox')).toEqual({
      view: 'inbox',
      params: { project: 'tapestry' },
    })
  })
})
