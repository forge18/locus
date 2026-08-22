import { For } from 'solid-js'

export interface SkeletonRowsProps {
  /** How many placeholder rows to draw. */
  count: number
  /**
   * The real row height, in pixels. It is required because the point of a
   * skeleton is that nothing reflows when the data arrives — a guessed height
   * moves the table twice instead of once.
   */
  rowHeight: number
  /** Column widths as CSS grid track sizes, so the bars land where the data will. */
  columns?: string[]
}

export function SkeletonRows(props: SkeletonRowsProps) {
  const columns = () => props.columns ?? ['1fr']
  return (
    <div class="skeleton-rows" aria-hidden="true" data-testid="skeleton-rows">
      <For each={Array.from({ length: props.count })}>
        {() => (
          <div
            class="skeleton-row"
            style={{
              height: `${props.rowHeight}px`,
              display: 'grid',
              'grid-template-columns': columns().join(' '),
              'align-items': 'center',
              gap: 'var(--g-4)',
              'border-bottom': '1px solid var(--border-subtle)',
            }}
          >
            <For each={columns()}>
              {() => (
                <span
                  class="skeleton-bar pulse"
                  style={{
                    height: '7px',
                    'border-radius': '3px',
                    background: 'var(--surface-selected)',
                  }}
                />
              )}
            </For>
          </div>
        )}
      </For>
    </div>
  )
}
