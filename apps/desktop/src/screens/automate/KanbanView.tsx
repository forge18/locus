import { For } from 'solid-js'
import { TaskCard } from './TaskCard'
import { Icon } from '../../ui/Icon'
import { Tag } from '../../ui/Tag'
import {
  BLOCKED_NOTE,
  COLUMN_LABELS,
  COLUMN_ORDER,
  HEADER_NOTE,
  useEvidence,
  useTasksByColumn,
} from '../../data/board'
import { useProjects } from '../../data/core'

/**
 * Six fixed columns, the same in every project. There is no add-column
 * affordance, and that is the feature: a column that means one thing in one
 * project and another somewhere else is not a column, it is a label.
 */
export interface KanbanViewProps {
  /** The list is the agent-session fixture; routing stays with the shell. */
  onShowAgents?: () => void
}

export function KanbanView(props: KanbanViewProps) {
  const byColumn = useTasksByColumn()

  return (
    <div class="kanban" data-testid="kanban">
      <div class="automate-view-switcher" data-testid="automate-view-switcher">
        <button
          type="button"
          class="automate-view-tab"
          data-testid="automate-kanban-tab"
          aria-pressed="true"
        >
          Kanban
        </button>
        <button
          type="button"
          class="automate-view-tab"
          data-testid="automate-list-tab"
          aria-pressed="false"
          onClick={() => props.onShowAgents?.()}
        >
          List
        </button>
      </div>

      <header class="kanban-head" data-testid="kanban-head">
        <span class="kanban-head-title" data-testid="kanban-title">
          {HEADER_NOTE}
        </span>
        <span class="kanban-note" data-testid="kanban-blocked-note">
          <Icon name="prohibit-inset" size={11} />
          {BLOCKED_NOTE}
        </span>
        <div class="kanban-chips" data-testid="kanban-chips">
          <For each={useProjects()}>
            {(project) => <Tag variant="neutral">{project.name}</Tag>}
          </For>
        </div>
      </header>

      <div class="kanban-columns" data-testid="kanban-columns">
        <For each={COLUMN_ORDER}>
          {(column) => (
            <section class="kanban-column" data-testid={`kanban-column-${column}`}>
              <div
                class={[
                  'kanban-column-head',
                  column === 'waiting_for_approval' ? 'kanban-column-head-approval' : '',
                ]
                  .filter(Boolean)
                  .join(' ')}
                data-testid={`kanban-column-head-${column}`}
              >
                {COLUMN_LABELS[column]}
                <span class="kanban-column-count" data-testid={`kanban-count-${column}`}>
                  {byColumn[column].length}
                </span>
              </div>
              <For each={byColumn[column]}>
                {(task) => <TaskCard task={task} evidence={useEvidence(task.id)} />}
              </For>
            </section>
          )}
        </For>
      </div>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default KanbanView
