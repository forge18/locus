import { For } from 'solid-js'
import { V2_GLOBAL_ROUTE_KINDS, V2_PROJECT_ROUTE_KINDS } from '../nav/v2-route-kinds'

export interface ProjectRailProps {
  selectedProject: string
}

export function ProjectRail(props: ProjectRailProps) {
  return (
    <nav aria-label="Application navigation" class="project-rail">
      <div data-testid="global-rail-routes">
        <For each={V2_GLOBAL_ROUTE_KINDS}>{(route) => <button type="button">{route.label}</button>}</For>
      </div>
      <section data-testid="selected-project-card">
        <span>{props.selectedProject}</span>
        <div data-testid="project-rail-routes">
          <For each={V2_PROJECT_ROUTE_KINDS}>{(route) => <button type="button">{route.label}</button>}</For>
        </div>
      </section>
    </nav>
  )
}
