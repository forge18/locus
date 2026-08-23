import { createSignal, onCleanup } from 'solid-js'
import type { JSX } from 'solid-js'

/**
 * A pane with a drag handle on one edge. Product, not chrome — Kobalte ships no
 * split panes and is right not to, because what a pane is depends on what is in it.
 *
 * The mockup's widths are the defaults; this is what makes them defaults rather
 * than constants.
 */
export interface ResizableProps {
  /** Preferred width in px, as the design draws it. */
  width: number
  min?: number
  max?: number
  /** Which edge carries the handle. */
  side: 'left' | 'right'
  class?: string
  testId?: string
  dataChangedFileState?: string
  children: JSX.Element
}

export function Resizable(props: ResizableProps) {
  /**
   * Null until dragged. Before that the pane is `clamp(min, preferred, max)` and
   * flexes with the host; after, it is the width the reader chose. The drawn
   * number is a preference either way, never a constant that forces a scrollbar.
   */
  const [dragged, setDragged] = createSignal<number | null>(null)
  let start = 0
  let startWidth = 0
  let el: HTMLDivElement | undefined

  const clamp = (n: number) =>
    Math.min(props.max ?? 640, Math.max(props.min ?? 160, n))

  const onMove = (e: PointerEvent) => {
    const delta = e.clientX - start
    setDragged(clamp(startWidth + (props.side === 'right' ? delta : -delta)))
  }

  const stop = () => {
    document.removeEventListener('pointermove', onMove)
    document.removeEventListener('pointerup', stop)
  }

  const onPointerDown = (e: PointerEvent) => {
    start = e.clientX
    // The rendered width once laid out; the preference before that.
    startWidth = dragged() ?? (el?.getBoundingClientRect().width || props.width)
    document.addEventListener('pointermove', onMove)
    document.addEventListener('pointerup', stop)
  }

  onCleanup(stop)

  return (
    <div
      ref={el}
      class={['resizable', props.class ?? ''].filter(Boolean).join(' ')}
      data-testid={props.testId ?? 'resizable'}
      data-side={props.side}
      data-dragged={dragged() === null ? undefined : 'true'}
      data-changed-file-state={props.dataChangedFileState}
      /* The clamp lives in the stylesheet; these are just the three numbers it
         needs. Dragging moves the preferred one, which the clamp still bounds. */
      style={{
        '--pane-min': `${props.min ?? 160}px`,
        '--pane-max': `${props.max ?? 640}px`,
        '--pane-w': `${dragged() ?? props.width}px`,
      }}
    >
      {props.children}
      <div
        class={`resize-handle resize-handle-${props.side}`}
        data-testid={`${props.testId ?? 'resizable'}-handle`}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize"
        onPointerDown={onPointerDown}
      />
    </div>
  )
}
