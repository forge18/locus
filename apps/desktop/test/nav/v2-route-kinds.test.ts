import { describe, expect, it } from 'vitest'
import { V2_FIXTURE_ROUTES } from '../../src/fixtures/v2-screen-inventory'
import {
  V2_GLOBAL_ROUTE_KINDS,
  V2_PROJECT_ROUTE_KINDS,
  V2_ROUTE_KINDS,
} from '../../src/nav/v2-route-kinds'

describe('nav/v2-route-kinds', () => {
  it('registers every fixture route with its declared global or project scope', () => {
    expect(V2_ROUTE_KINDS.map((route) => route.id)).toEqual(
      V2_FIXTURE_ROUTES.map((route) => route.id),
    )
    expect(V2_ROUTE_KINDS.map((route) => route.scope)).toEqual(
      V2_FIXTURE_ROUTES.map((route) => route.scope),
    )
    expect(new Set(V2_ROUTE_KINDS.map((route) => route.id)).size).toBe(31)
  })

  it('provides non-empty scope collections derived from the registered routes', () => {
    expect(V2_GLOBAL_ROUTE_KINDS).toEqual(
      V2_ROUTE_KINDS.filter((route) => route.scope === 'global'),
    )
    expect(V2_PROJECT_ROUTE_KINDS).toEqual(
      V2_ROUTE_KINDS.filter((route) => route.scope === 'project'),
    )
  })
})
