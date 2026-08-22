import { Show } from 'solid-js'
import type { JSX } from 'solid-js'
import { Icon } from './Icon'

export interface InlineErrorProps {
  /** What failed. */
  cause: string
  /** What the reader can do about it. Required — an error with no next step is a dead end. */
  next: string
  /** Optional control that performs the next step. */
  action?: JSX.Element
}

/**
 * Errors render on the surface that failed, never as a toast. The reader is
 * already looking at the thing; telling them somewhere else is a second search.
 */
export function InlineError(props: InlineErrorProps) {
  return (
    <div
      class="inline-error"
      role="alert"
      data-testid="inline-error"
      style={{
        display: 'flex',
        'align-items': 'flex-start',
        gap: 'var(--g-3)',
        padding: 'var(--g-4)',
        'border-radius': 'var(--r-card)',
        background: 'var(--surface-raised)',
        'box-shadow': 'inset 0 0 0 1px var(--status-danger)',
        color: 'var(--status-danger)',
      }}
    >
      <Icon name="warning-octagon" weight="fill" size={13} style={{ 'flex-shrink': 0, 'margin-top': '1px' }} />
      <div style={{ display: 'flex', 'flex-direction': 'column', gap: '3px' }}>
        <span class="t-body em" data-testid="inline-error-cause">{props.cause}</span>
        <span class="t-meta" style={{ color: 'var(--text-secondary)' }} data-testid="inline-error-next">
          {props.next}
        </span>
      </div>
      <Show when={props.action}>
        <div style={{ 'margin-left': 'auto' }}>{props.action}</div>
      </Show>
    </div>
  )
}
