export interface RunningPillProps {
  running: number
  needsYou: number
}

export function RunningPill(props: RunningPillProps) {
  return (
    <button class="running-count" data-testid="running-pill" type="button">
      <span class="live-dot pulse" data-testid="running-pill-dot" />
      <span>{props.running} running</span>
      <span>{props.needsYou} needs you</span>
    </button>
  )
}
