import {
  Desktop_FIXTURE_ROUTES,
  type FixtureScope,
  type DesktopFixtureRoute,
} from '../fixtures/desktop-screen-inventory'

export interface DesktopRouteKind {
  id: DesktopFixtureRoute['id']
  label: DesktopFixtureRoute['label']
  scope: FixtureScope
  category: DesktopFixtureRoute['category']
}

function routeKind(route: DesktopFixtureRoute): DesktopRouteKind {
  return Object.freeze({ id: route.id, label: route.label, scope: route.scope, category: route.category })
}

/** The one current 29-view route inventory. */
export const Desktop_ROUTE_KINDS = Object.freeze(Desktop_FIXTURE_ROUTES.map(routeKind))
export const Desktop_ALL_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === 'all'),
)
/** Compatibility name for callers that only need the cross-project subset. */
export const Desktop_GLOBAL_ROUTE_KINDS = Desktop_ALL_ROUTE_KINDS

export const Desktop_APP_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === 'app'),
)
export const Desktop_PROJECT_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === 'project'),
)
