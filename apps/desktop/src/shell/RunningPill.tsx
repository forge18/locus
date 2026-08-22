import { For, Show, createSignal } from 'solid-js'

export interface ActiveSession {
  id: string
  label: string
  needsAttention: boolean
  lastActivityAt: number
}

export interface RunningPillProps {
  running: number
  needsYou: number
  sessions?: ActiveSession[]
}

function orderedSessions(sessions: ActiveSession[]): ActiveSession[] {
  return [...sessions].sort(
    (a, b) => Number(b.needsAttention) - Number(a.needsAttention) || b.lastActivityAt - a.lastActivityAt,
  )
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
        <span data-testid="running-count" aria-live="polite">{props.running} running</span>
        <span data-testid="needs-you-count" aria-live="assertive">{props.needsYou} needs you</span>
      </button>
      <Show when={open()}>
        <div aria-label="Active sessions" class="running-popover" role="dialog">
          <ul data-testid="active-session-list">
            <For each={orderedSessions(props.sessions ?? [])}>{(session) => <li>{session.label}</li>}</For>
          </ul>
          <button aria-label="Close active sessions" onClick={() => setOpen(false)} type="button">
            Close
          </button>
        </div>
      </Show>
    </div>
  )
}
