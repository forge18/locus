import { For, createMemo, createSignal, onCleanup, onMount } from 'solid-js'
import type { JSX } from 'solid-js'

/**
 * A fixed-height windowed list.
 *
 * Product, not chrome: what a row is and how tall it is belongs to the surface
 * showing it, which is why this is not in `src/ui/`. The height is required
 * rather than measured — the tables that need this already declare one, and a
 * measured virtualizer trades determinism for a flexibility no table here wants.
 *
 * Rows above and below the window are replaced by two spacers, so the scrollbar
 * stays the size of the whole list and nothing about scrolling feels windowed.
 */
export interface VirtualRowsProps<T> {
  items: T[]
  /** Real row height in px. The list is only correct if this is the real one. */
  rowHeight: number
  /** Visible height of the scroll container in px. */
  height: number
  /** Rows kept rendered beyond each edge, so a fast scroll does not flash empty. */
  overscan?: number
  /** How many rows exist in total, when more can still be fetched. */
  total?: number
  /**
   * Called when the window comes within `overscan` rows of what is loaded. The
   * caller fetches the next page; this component never fetches anything itself.
   */
  onLoadMore?: () => void
  children: (item: T, index: number) => JSX.Element
  class?: string
  testId?: string
}

const DEFAULT_OVERSCAN = 8

export function VirtualRows<T>(props: VirtualRowsProps<T>) {
  const [scrollTop, setScrollTop] = createSignal(0)
  let viewport: HTMLDivElement | undefined

  const overscan = () => props.overscan ?? DEFAULT_OVERSCAN
  const total = () => props.total ?? props.items.length

  const first = createMemo(() =>
    Math.max(0, Math.floor(scrollTop() / props.rowHeight) - overscan()),
  )
  const visible = createMemo(() => Math.ceil(props.height / props.rowHeight) + overscan() * 2)
  const last = createMemo(() => Math.min(props.items.length, first() + visible()))

  const window_ = createMemo(() => props.items.slice(first(), last()))

  const onScroll = () => {
    if (!viewport) return
    setScrollTop(viewport.scrollTop)
    // Within an overscan of the end of what is loaded: ask for the next page.
    if (props.onLoadMore && last() >= props.items.length - overscan() && props.items.length < total()) {
      props.onLoadMore()
    }
  }

  onMount(() => viewport?.addEventListener('scroll', onScroll))
  onCleanup(() => viewport?.removeEventListener('scroll', onScroll))

  return (
    <div
      ref={viewport}
      class={['virtual-rows', props.class ?? ''].filter(Boolean).join(' ')}
      data-testid={props.testId ?? 'virtual-rows'}
      data-total={total()}
      data-loaded={props.items.length}
      data-first={first()}
      data-last={last()}
      style={{ height: `${props.height}px`, overflow: 'auto' }}
    >
      {/* The whole list's height, so the scrollbar tells the truth. */}
      <div style={{ height: `${first() * props.rowHeight}px` }} data-testid="virtual-spacer-top" />
      <For each={window_()}>{(item, i) => props.children(item, first() + i())}</For>
      <div
        style={{ height: `${Math.max(0, total() - last()) * props.rowHeight}px` }}
        data-testid="virtual-spacer-bottom"
      />
    </div>
  )
}
