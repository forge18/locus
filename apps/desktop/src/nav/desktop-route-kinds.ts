import {
  DESKTOP_ROUTES,
  type DesktopRouteScope,
  type DesktopRoute,
} from "./desktop-screen-inventory";

export interface DesktopRouteKind {
  id: DesktopRoute["id"];
  label: DesktopRoute["label"];
  scope: DesktopRouteScope;
  category: DesktopRoute["category"];
}

function routeKind(route: DesktopRoute): DesktopRouteKind {
  return Object.freeze({
    id: route.id,
    label: route.label,
    scope: route.scope,
    category: route.category,
  });
}

/** The one current route inventory. */
export const Desktop_ROUTE_KINDS = Object.freeze(DESKTOP_ROUTES.map(routeKind));
export const Desktop_ALL_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === "all"),
);
/** Kept as an empty compatibility export; `all` is the canonical cross-project scope. */
export const Desktop_GLOBAL_ROUTE_KINDS = Object.freeze(
  [] as DesktopRouteKind[],
);

export const Desktop_APP_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === "app"),
);
export const Desktop_PROJECT_ROUTE_KINDS = Object.freeze(
  Desktop_ROUTE_KINDS.filter((route) => route.scope === "project"),
);
