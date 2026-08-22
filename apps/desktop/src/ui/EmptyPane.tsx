import { Show } from 'solid-js'
import type { JSX } from 'solid-js'
import { Icon } from './Icon'

export interface EmptyPaneProps {
  /**
   * Why this pane is empty, as a sentence. Required, and deliberately so:
   * "No agent has run today" and "Nothing needs you" are different facts, and
   * "No items" is neither of them.
   */
  reason: string
  /** Optional Phosphor icon name to sit above the reason. */
  icon?: string
  /** An action the reader can take from here, if there is one. */
  action?: JSX.Element
}

export function EmptyPane(props: EmptyPaneProps) {
  return (
    <div
      class="empty-pane"
      data-testid="empty-pane"
      style={{
        display: 'flex',
        'flex-direction': 'column',
        'align-items': 'center',
        'justify-content': 'center',
        gap: 'var(--g-4)',
        padding: '24px',
        height: '100%',
        color: 'var(--text-secondary)',
        'text-align': 'center',
      }}
    >
      <Show when={props.icon}>
        <Icon name={props.icon!} size={19} style={{ color: 'var(--text-muted)' }} />
      </Show>
      <p class="t-body" style={{ margin: 0, 'max-width': '38ch' }}>
        {props.reason}
      </p>
      <Show when={props.action}>{props.action}</Show>
    </div>
  )
}
