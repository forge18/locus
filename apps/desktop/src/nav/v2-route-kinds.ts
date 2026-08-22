import {
  V2_FIXTURE_ROUTES,
  type FixtureScope,
  type V2FixtureRoute,
} from '../fixtures/v2-screen-inventory'

export interface V2RouteKind {
  id: V2FixtureRoute['id']
  label: V2FixtureRoute['label']
  scope: FixtureScope
}

function routeKind(route: V2FixtureRoute): V2RouteKind {
  return Object.freeze({ id: route.id, label: route.label, scope: route.scope })
}

/** The v2 resolver's route kinds, derived from the delivered fixture inventory. */
export const V2_ROUTE_KINDS = Object.freeze(V2_FIXTURE_ROUTES.map(routeKind))

export const V2_GLOBAL_ROUTE_KINDS = Object.freeze(
  V2_ROUTE_KINDS.filter((route) => route.scope === 'global'),
)

export const V2_PROJECT_ROUTE_KINDS = Object.freeze(
  V2_ROUTE_KINDS.filter((route) => route.scope === 'project'),
)
