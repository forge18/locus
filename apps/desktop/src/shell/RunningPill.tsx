import { Show, createSignal } from 'solid-js'

export interface RunningPillProps {
  running: number
  needsYou: number
}

export function RunningPill(props: RunningPillProps) {
  const [open, setOpen] = createSignal(false)

  return (
    <div class="running-pill-wrap">
      <button
        aria-expanded={open()}
        class="running-count"
        data-testid="running-pill"
        onClick={() => setOpen(!open())}
        type="button"
      >
        <span class="live-dot pulse" data-testid="running-pill-dot" />
        <span>{props.running} running</span>
        <span>{props.needsYou} needs you</span>
      </button>
      <Show when={open()}>
        <div aria-label="Active sessions" class="running-popover" role="dialog">
          <button aria-label="Close active sessions" onClick={() => setOpen(false)} type="button">
            Close
          </button>
        </div>
      </Show>
    </div>
  )
}
