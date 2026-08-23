import { describe, expect, it } from 'vitest'
import { Desktop_FIXTURE_ROUTES } from '../../src/fixtures/desktop-screen-inventory'
import {
  Desktop_GLOBAL_ROUTE_KINDS,
  Desktop_PROJECT_ROUTE_KINDS,
  Desktop_ROUTE_KINDS,
} from '../../src/nav/desktop-route-kinds'

describe('nav/desktop-route-kinds', () => {
  it('registers every fixture route with its declared global or project scope', () => {
    expect(Desktop_ROUTE_KINDS.map((route) => route.id)).toEqual(
      Desktop_FIXTURE_ROUTES.map((route) => route.id),
    )
    expect(Desktop_ROUTE_KINDS.map((route) => route.scope)).toEqual(
      Desktop_FIXTURE_ROUTES.map((route) => route.scope),
    )
    expect(new Set(Desktop_ROUTE_KINDS.map((route) => route.id)).size).toBe(31)
  })

  it('provides non-empty scope collections derived from the registered routes', () => {
    expect(Desktop_GLOBAL_ROUTE_KINDS).toEqual(
      Desktop_ROUTE_KINDS.filter((route) => route.scope === 'global'),
    )
    expect(Desktop_PROJECT_ROUTE_KINDS).toEqual(
      Desktop_ROUTE_KINDS.filter((route) => route.scope === 'project'),
    )
  })
})
