import type { SessionDetail } from '../../data/sessions'

export interface SessionCardProps {
  session: SessionDetail
  selected: boolean
  onSelect: () => void
}

export function SessionCard(props: SessionCardProps) {
  const status = () => props.session.status

  return (
    <button
      type="button"
      class={['session-card', status() === 'stuck' ? 'session-card-stuck' : '']
        .filter(Boolean)
        .join(' ')}
      data-testid={`session-card-${props.session.id}`}
      data-status={status()}
      aria-selected={props.selected ? 'true' : 'false'}
      onClick={props.onSelect}
    >
      <div class="session-card-top">
        <span
          class={`session-dot session-dot-${status()}`}
          data-testid={`session-dot-${props.session.id}`}
        />
        <span class="session-project">{props.session.project}</span>
        <span class="session-agent">{props.session.agent}</span>
        <span class="session-role">{props.session.role}</span>
        <span class="session-tokens" data-testid={`session-tokens-${props.session.id}`}>
          {/* Unknown is not zero: a harness that reports nothing gets the word. */}
          {props.session.tokens ?? 'unknown'}
        </span>
      </div>

      <div class="session-task" data-testid={`session-task-${props.session.id}`}>
        {props.session.task}
      </div>

      <div class="session-bottom">
        <span
          class={[
            'session-chip',
            status() === 'stuck' ? 'session-chip-stuck' : '',
            status() === 'waiting' ? 'session-chip-waiting' : '',
          ]
            .filter(Boolean)
            .join(' ')}
          data-testid={`session-chip-${props.session.id}`}
        >
          {status()}
        </span>
        <span class="session-tool" data-testid={`session-tool-${props.session.id}`}>
          {props.session.tool ?? 'no tool'}
        </span>
        <span class="session-runs" data-testid={`session-runs-${props.session.id}`}>
          {props.session.runs} {props.session.runs === 1 ? 'run' : 'runs'}
        </span>
      </div>
    </button>
  )
}
