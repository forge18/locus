import { For, Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import type { AttentionKind, AttentionRow } from '../../data/status'

export interface WantsAttentionProps {
  rows: AttentionRow[]
  onAction?: (row: AttentionRow) => void
}

/**
 * waiting is not idle. One is blocked on a gate somebody has to open; the other is
 * producing no events at all, and confusing them wastes a person on the wrong one.
 */
const KIND_ICON: Record<AttentionKind, { name: string; weight: 'regular' | 'fill'; color: string }> = {
  stuck: { name: 'warning-octagon', weight: 'fill', color: 'var(--bad)' },
  idle: { name: 'moon', weight: 'regular', color: 'var(--mu)' },
  waiting: { name: 'hourglass-medium', weight: 'regular', color: 'var(--mu)' },
}

export function WantsAttention(props: WantsAttentionProps) {
  return (
    <section class="panel" data-testid="wants-attention">
      <span class="panel-title">Wants attention</span>
      <For each={props.rows}>
        {(row) => (
          <div
            class={['attention-row', row.kind === 'stuck' ? 'attention-row-stuck' : '']
              .filter(Boolean)
              .join(' ')}
            data-testid={`attention-${row.kind}`}
            data-kind={row.kind}
          >
            <Icon
              name={KIND_ICON[row.kind].name}
              weight={KIND_ICON[row.kind].weight}
              size={13}
              style={{ color: KIND_ICON[row.kind].color }}
            />
            <div>
              <div class="attention-subject">{row.subject}</div>
              <div class="attention-detail" data-testid={`attention-${row.kind}-detail`}>
                {row.detail}
              </div>
            </div>
            <Show when={row.action}>
              <button
                type="button"
                class="attention-action"
                data-testid={`attention-${row.kind}-action`}
                onClick={() => props.onAction?.(row)}
              >
                {row.action}
              </button>
            </Show>
          </div>
        )}
      </For>
    </section>
  )
}
