import {
  Desktop_FIXTURE_ROUTES,
  type FixtureScope,
  type DesktopFixtureRoute,
} from '../fixtures/desktop-screen-inventory'

export interface DesktopRouteKind {
  id: DesktopFixtureRoute['id']
  label: DesktopFixtureRoute['label']
  scope: FixtureScope
}

function routeKind(route: DesktopFixtureRoute): DesktopRouteKind {
  return Object.freeze({ id: route.id, label: route.label, scope: route.scope })
}

/** The desktop resolver's route kinds, derived from the delivered fixture inventory. */
export const Desktop_ROUTE_KINDS = Object.freeze(Desktop_FIXTURE_ROUTES.map(routeKind))

export const Desktop_GLOBAL_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === 'global'),
)

export const Desktop_PROJECT_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === 'project'),
)
