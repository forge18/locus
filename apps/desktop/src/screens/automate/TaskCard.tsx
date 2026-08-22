import { Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import { APPROVAL_NOTE } from '../../data/board'
import type { Task } from '../../data/board'

export interface TaskCardProps {
  task: Task
  /** Set on a Done card: what proves it. */
  evidence?: { runs: number; events: number } | null
}

/**
 * A card carries its status on itself. `blocked` is drawn here, in place, rather
 * than by moving the card — because blocked is orthogonal to progress and a
 * blocked-column would lose which stage it was blocked *at*.
 */
export function TaskCard(props: TaskCardProps) {
  const classes = () =>
    [
      'task-card',
      props.task.status === 'stuck' ? 'task-card-stuck' : '',
      props.task.column === 'waiting_for_approval' ? 'task-card-approval' : '',
      props.task.column === 'done' ? 'task-card-done' : '',
    ]
      .filter(Boolean)
      .join(' ')

  return (
    <article
      class={classes()}
      data-testid={`task-card-${props.task.id}`}
      data-status={props.task.status}
      data-column={props.task.column}
    >
      <div class="task-card-title">
        <Show when={props.task.status === 'blocked'}>
          <Icon
            name="prohibit-inset"
            size={12}
            style={{ color: 'var(--status-danger)', 'flex-shrink': 0, 'margin-top': '1px' }}
            label="Blocked"
          />
        </Show>
        <span>{props.task.title}</span>
      </div>

      <div class="task-card-meta">
        <span>
          <span class="task-card-project">{props.task.projectId.replace(/^p-/, '')}</span> ·{' '}
          {props.task.repoId.replace(/^r-/, '')}
        </span>
        <span class="mono">{props.task.verifyCommand}</span>
        <Show when={props.task.assignee}>
          <span>
            {props.task.assignee} · {props.task.tools}
          </span>
        </Show>
        <span>Gate: {props.task.gate}</span>

        <Show when={props.task.status === 'stuck'}>
          <span class="task-card-stuck-line" data-testid={`task-stuck-${props.task.id}`}>
            stuck {props.task.stuckIterations}/{props.task.maxIterations} · {props.task.tokens}
          </span>
        </Show>
        <Show when={props.task.column === 'waiting_for_approval'}>
          <span class="task-card-approval-line" data-testid={`task-approval-${props.task.id}`}>
            {APPROVAL_NOTE}
          </span>
        </Show>
        <Show when={props.evidence}>
          <span class="task-card-evidence" data-testid={`task-evidence-${props.task.id}`}>
            evidence: {props.evidence!.runs} runs, {props.evidence!.events} events
          </span>
        </Show>
      </div>
    </article>
  )
}
