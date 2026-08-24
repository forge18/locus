import { createSignal, type Accessor } from 'solid-js'
import type { DesktopRouteKind } from './desktop-route-kinds'

export type RouteScope =
  | { kind: 'all' }
  | { kind: 'app' }
  | { kind: 'project'; project: string }

export interface RouteScopeStore {
  scope: Accessor<RouteScope>
  showAll: () => void
  /** @deprecated use showAll; retained for the pre-M0.7 API. */
  showGlobal: () => void
  showApp: () => void
  showProject: (project: string) => void
}

export function resolveRouteScope(route: DesktopRouteKind, selectedProject: string | null): RouteScope {
  if (route.scope === 'all') return { kind: 'all' }
  if (route.scope === 'app') return { kind: 'app' }
  const project = selectedProject?.trim()
  if (!project) throw new Error(`${route.label} requires a selected project`)
  return { kind: 'project', project }
}

export function createRouteScope(initialProject: string): RouteScopeStore {
  const [scope, setScope] = createSignal<RouteScope>({ kind: 'project', project: initialProject })
  return {
    scope,
    showAll: () => setScope({ kind: 'all' }),
    showGlobal: () => setScope({ kind: 'all' }),
    showApp: () => setScope({ kind: 'app' }),
    showProject: (project) => setScope({ kind: 'project', project }),
  }
}
