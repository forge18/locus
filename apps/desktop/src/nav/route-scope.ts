import { createSignal, type Accessor } from 'solid-js'

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

/** Scope is explicit: global routes never carry a hidden selected project. */
export function createRouteScope(initialProject: string): RouteScopeStore {
  const [scope, setScope] = createSignal<RouteScope>({ kind: 'project', project: initialProject })

  return {
    scope,
    showGlobal: () => setScope({ kind: 'global' }),
    showProject: (project) => setScope({ kind: 'project', project }),
  }
}
