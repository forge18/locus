import { describe, expect, it } from 'vitest'
import { resolveRouteScope } from '../../src/nav/route-scope'
import { Desktop_GLOBAL_ROUTE_KINDS, Desktop_PROJECT_ROUTE_KINDS } from '../../src/nav/desktop-route-kinds'

describe('nav/project-route-requires-project', () => {
  it('rejects a project-scoped route with no selected project', () => {
    expect(() => resolveRouteScope(Desktop_PROJECT_ROUTE_KINDS[0], null)).toThrow(
      `${Desktop_PROJECT_ROUTE_KINDS[0].label} requires a selected project`,
    )
  })

  it('allows a global route without a selected project', () => {
    expect(resolveRouteScope(Desktop_GLOBAL_ROUTE_KINDS[0], null)).toEqual({ kind: 'global' })
  })
})
