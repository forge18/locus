import { Desktop_ROUTE_KINDS } from "./desktop-route-kinds";
import type { DesktopRouteScope } from "./desktop-screen-inventory";

export type DesktopRouteId = (typeof Desktop_ROUTE_KINDS)[number]["id"];
export type DesktopLocatorScope = DesktopRouteScope;

export interface DesktopNavTarget {
  route: DesktopRouteId;
  scope:
    | { kind: "all" }
    | { kind: "app" }
    | { kind: "project"; project: string };
  botId?: string;
}

const LOCATOR_SCHEME = "locus://";
const SEGMENT = /^[A-Za-z0-9._@-]+$/;

export class DesktopLocatorError extends Error {}

function routeFor(id: string): (typeof Desktop_ROUTE_KINDS)[number] {
  const route = Desktop_ROUTE_KINDS.find(
    (candidate: (typeof Desktop_ROUTE_KINDS)[number]) => candidate.id === id,
  );
  if (!route)
    throw new DesktopLocatorError(
      `route: "${id}" is not a registered desktop route`,
    );
  return route;
}

function projectFor(project: string | undefined): string {
  if (
    !project ||
    !SEGMENT.test(project) ||
    project === "all" ||
    project === "app"
  ) {
    throw new DesktopLocatorError(
      `project: "${project}" is not a project segment`,
    );
  }
  return project;
}

/**
 * Formats the canonical view locator. Project names are the first segment;
 * cross-project and install-wide views use the reserved `all` and `app` scopes.
 */
export function formatDesktopLocator(
  routeId: DesktopRouteId,
  project?: string,
  botId?: string,
): string {
  const route = routeFor(routeId);
  if (route.id === "workers" && botId !== undefined) {
    const scope = projectFor(project);
    if (!SEGMENT.test(botId) || !botId) {
      throw new DesktopLocatorError(`worker: "${botId}" is not a worker segment`);
    }
    return `${LOCATOR_SCHEME}${scope}/workers/${botId}`;
  }
  if (route.scope === "project")
    return `${LOCATOR_SCHEME}${projectFor(project)}/view/${route.id}`;
  if (project !== undefined) {
    throw new DesktopLocatorError(
      `scope: ${route.scope} route "${routeId}" does not carry a project`,
    );
  }
  return `${LOCATOR_SCHEME}${route.scope}/view/${route.id}`;
}

/** Resolves every canonical desktop locator through this one boundary. */
export function resolveDesktopLocator(locator: string): DesktopNavTarget {
  if (!locator.startsWith(LOCATOR_SCHEME)) {
    throw new DesktopLocatorError(`scheme: expected "${LOCATOR_SCHEME}"`);
  }
  const segments = locator.slice(LOCATOR_SCHEME.length).split("/");
  if (segments.length === 3 && segments[1] === "workers") {
    const project = projectFor(segments[0]);
    const workerId = segments[2];
    if (!workerId || !SEGMENT.test(workerId)) {
      throw new DesktopLocatorError(`worker: "${workerId}" is not a worker segment`);
    }
    return { route: "workers", scope: { kind: "project", project }, botId: workerId };
  }
  if (segments.length !== 3 || segments[1] !== "view") {
    throw new DesktopLocatorError(
      "locator: expected locus://<project|all|app>/view/<route> or locus://<project>/workers/<worker-id>",
    );
  }
  const [scope, , id] = segments;
  const route = routeFor(id);

  if (scope === "all" || scope === "app") {
    if (route.scope !== scope) {
      throw new DesktopLocatorError(
        `scope: route "${route.id}" requires ${route.scope} scope`,
      );
    }
    return { route: route.id, scope: { kind: scope } };
  }

  const project = projectFor(scope);
  if (route.scope !== "project") {
    throw new DesktopLocatorError(
      `scope: route "${route.id}" is ${route.scope}`,
    );
  }
  return { route: route.id, scope: { kind: "project", project } };
}
