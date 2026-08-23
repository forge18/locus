import { createSignal, type Accessor } from 'solid-js'
import type { DesktopRouteKind } from './desktop-route-kinds'

export type RouteScope =
  | { kind: 'global' }
  | {
      kind: 'project'
      project: string
    }

export interface RouteScopeStore {
  scope: Accessor<RouteScope>
  showGlobal: () => void
  showProject: (project: string) => void
}

export function resolveRouteScope(route: DesktopRouteKind, selectedProject: string | null): RouteScope {
  if (route.scope === 'global') return { kind: 'global' }

  const project = selectedProject?.trim()
  if (!project) throw new Error(`${route.label} requires a selected project`)
  return { kind: 'project', project }
}

/** Scope is explicit: global routes never carry a hidden selected project. */
export function createRouteScope(initialProject: string): RouteScopeStore {
  const [scope, setScope] = createSignal<RouteScope>({ kind: 'project', project: initialProject })

  return {
    scope,
    showGlobal: () => setScope({ kind: 'global' }),
    showProject: (project) => setScope({ kind: 'project', project }),
  }
}
