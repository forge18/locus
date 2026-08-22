import { createMemo, createSignal, For } from 'solid-js'
import { V2_GLOBAL_ROUTE_KINDS, V2_PROJECT_ROUTE_KINDS } from '../nav/v2-route-kinds'

export interface ProjectRailProps {
  selectedProject: string
  inboxCount?: number
  projects?: readonly string[]
}

export function ProjectRail(props: ProjectRailProps) {
  const [filter, setFilter] = createSignal('')
  const projects = createMemo(() => {
    const needle = filter().trim().toLowerCase()
    return (props.projects ?? [props.selectedProject]).filter((project) => project.toLowerCase().includes(needle))
  })

  return (
    <nav aria-label="Application navigation" class="project-rail" data-testid="project-rail">
      <div data-testid="global-rail-routes">
        <For each={V2_GLOBAL_ROUTE_KINDS}>
          {(route) => (
            <button type="button">
              {route.label}
              {route.id === 'inbox' && props.inboxCount ? (
                <span data-testid="global-rail-inbox-badge">{props.inboxCount}</span>
              ) : null}
            </button>
          )}
        </For>
      </div>
      <section data-testid="selected-project-card">
        <span>{props.selectedProject}</span>
        <input
          type="search"
          data-testid="project-switcher-filter"
          aria-label="Filter projects"
          value={filter()}
          onInput={(event) => setFilter(event.currentTarget.value)}
        />
        <div data-testid="project-switcher-results">
          <For each={projects()}>{(project) => <button type="button">{project}</button>}</For>
        </div>
        <div data-testid="project-rail-routes">
          <For each={V2_PROJECT_ROUTE_KINDS}>{(route) => <button type="button">{route.label}</button>}</For>
        </div>
      </section>
    </nav>
  )
}
