import { ContextMenu } from '../ui/ContextMenu'
import { Icon } from '../ui/Icon'

export interface ProjectFilterProps {
  projects: { id: string; name: string }[]
  /** The projects in scope. Empty means all of them. */
  selected: string[]
  onChange: (selected: string[]) => void
}

/**
 * A scope filter, never a switcher. Switching means leaving somewhere that was
 * still running, so this narrows what a screen shows and never navigates.
 */
export function ProjectFilter(props: ProjectFilterProps) {
  const isAll = () => props.selected.length === 0
  const label = () =>
    isAll()
      ? 'All projects'
      : props.selected.length === 1
        ? props.projects.find((p) => p.id === props.selected[0])!.name
        : `${props.selected.length} projects`

  const toggle = (id: string) => {
    const next = props.selected.includes(id)
      ? props.selected.filter((x) => x !== id)
      : [...props.selected, id]
    props.onChange(next)
  }

  return (
    <ContextMenu
      heading="Filter to"
      actions={[
        { label: 'All projects', onSelect: () => props.onChange([]) },
        ...props.projects.map((p) => ({ label: p.name, onSelect: () => toggle(p.id) })),
      ]}
    >
      <button class="project-filter" data-testid="project-filter" type="button">
        <Icon name="funnel" size={11} style={{ color: 'var(--action-attention)' }} />
        <span data-testid="project-filter-label">{label()}</span>
        <span class="project-filter-count" data-testid="project-filter-count">
          {props.projects.length}
        </span>
        <Icon name="caret-down" size={10} style={{ color: 'var(--text-muted)' }} />
      </button>
    </ContextMenu>
  )
}
