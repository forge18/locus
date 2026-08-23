import { Desktop_ROUTE_KINDS } from "./desktop-route-kinds";
import type { RouteScope } from "./route-scope";
import type { DesktopRouteKind } from "./desktop-route-kinds";

export type DesktopRouteId = DesktopRouteKind["id"];

export interface DesktopNavTarget {
  route: DesktopRouteId;
  scope: RouteScope;
}

const LOCATOR_SCHEME = "locus://";
const SEGMENT = /^[A-Za-z0-9._@-]+$/;

export class DesktopLocatorError extends Error {}

function routeFor(id: string): DesktopRouteKind {
  const route = Desktop_ROUTE_KINDS.find((candidate) => candidate.id === id);
  if (!route)
    throw new DesktopLocatorError(`route: "${id}" is not a registered desktop route`);
  return route;
}

function projectFor(project: string | undefined): string {
  if (!project || !SEGMENT.test(project)) {
    throw new DesktopLocatorError(`project: "${project}" is not a project segment`);
  }
  return project;
}

/** Formats the desktop canonical locator with an explicit global or project scope. */
export function formatDesktopLocator(routeId: DesktopRouteId, project?: string): string {
  const route = routeFor(routeId);
  if (route.scope === "global") {
    if (project !== undefined) {
      throw new DesktopLocatorError(
        `scope: global route "${routeId}" does not carry a project`,
      );
    }
    return `${LOCATOR_SCHEME}global/${routeId}`;
  }

  return `${LOCATOR_SCHEME}project/${projectFor(project)}/${routeId}`;
}

/** Resolves a canonical desktop locator. Legacy v1 locators remain with `resolve`. */
export function resolveDesktopLocator(locator: string): DesktopNavTarget {
  if (!locator.startsWith(LOCATOR_SCHEME)) {
    throw new DesktopLocatorError(
      `scheme: expected "${LOCATOR_SCHEME}", got "${locator.split("/")[0]}//"`,
    );
  }

  const [scope, ...tail] = locator.slice(LOCATOR_SCHEME.length).split("/");
  if (scope === "global") {
    if (tail.length !== 1) {
      throw new DesktopLocatorError(
        "scope: global locators are locus://global/<route>",
      );
    }
    const route = routeFor(tail[0]);
    if (route.scope !== "global") {
      throw new DesktopLocatorError(
        `scope: route "${route.id}" requires a project scope`,
      );
    }
    return { route: route.id, scope: { kind: "global" } };
  }

  if (scope === "project") {
    if (tail.length !== 2) {
      throw new DesktopLocatorError(
        "scope: project locators are locus://project/<project>/<route>",
      );
    }
    const project = projectFor(tail[0]);
    const route = routeFor(tail[1]);
    if (route.scope !== "project") {
      throw new DesktopLocatorError(`scope: route "${route.id}" is global`);
    }
    return { route: route.id, scope: { kind: "project", project } };
  }

  throw new DesktopLocatorError(
    `scope: expected "global" or "project", got "${scope}"`,
  );
}
