import { V2_ROUTE_KINDS } from './v2-route-kinds'
import type { RouteScope } from './route-scope'
import type { V2RouteKind } from './v2-route-kinds'

export type V2RouteId = V2RouteKind['id']

export interface V2NavTarget {
  route: V2RouteId
  scope: RouteScope
}

const LOCATOR_SCHEME = 'locus://'
const SEGMENT = /^[A-Za-z0-9._@-]+$/

export class V2LocatorError extends Error {}

function routeFor(id: string): V2RouteKind {
  const route = V2_ROUTE_KINDS.find((candidate) => candidate.id === id)
  if (!route) throw new V2LocatorError(`route: "${id}" is not a registered v2 route`)
  return route
}

function projectFor(project: string | undefined): string {
  if (!project || !SEGMENT.test(project)) {
    throw new V2LocatorError(`project: "${project}" is not a project segment`)
  }
  return project
}

/** Formats the v2 canonical locator with an explicit global or project scope. */
export function formatV2Locator(routeId: V2RouteId, project?: string): string {
  const route = routeFor(routeId)
  if (route.scope === 'global') {
    if (project !== undefined) {
      throw new V2LocatorError(`scope: global route "${routeId}" does not carry a project`)
    }
    return `${LOCATOR_SCHEME}global/${routeId}`
  }

  return `${LOCATOR_SCHEME}project/${projectFor(project)}/${routeId}`
}

/** Resolves a canonical v2 locator. Legacy v1 locators remain with `resolve`. */
export function resolveV2Locator(locator: string): V2NavTarget {
  if (!locator.startsWith(LOCATOR_SCHEME)) {
    throw new V2LocatorError(`scheme: expected "${LOCATOR_SCHEME}", got "${locator.split('/')[0]}//"`)
  }

  const [scope, ...tail] = locator.slice(LOCATOR_SCHEME.length).split('/')
  if (scope === 'global') {
    if (tail.length !== 1) {
      throw new V2LocatorError('scope: global locators are locus://global/<route>')
    }
    const route = routeFor(tail[0])
    if (route.scope !== 'global') {
      throw new V2LocatorError(`scope: route "${route.id}" requires a project scope`)
    }
    return { route: route.id, scope: { kind: 'global' } }
  }

  if (scope === 'project') {
    if (tail.length !== 2) {
      throw new V2LocatorError('scope: project locators are locus://project/<project>/<route>')
    }
    const project = projectFor(tail[0])
    const route = routeFor(tail[1])
    if (route.scope !== 'project') {
      throw new V2LocatorError(`scope: route "${route.id}" is global`)
    }
    return { route: route.id, scope: { kind: 'project', project } }
  }

  throw new V2LocatorError(`scope: expected "global" or "project", got "${scope}"`)
}
