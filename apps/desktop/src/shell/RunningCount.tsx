export interface RunningCountProps {
  count: number
}

/** The pulsing dot is the only thing in the title bar that moves. */
export function RunningCount(props: RunningCountProps) {
  return (
    <div class="running-count" data-testid="running-count">
      <span class="live-dot pulse" data-testid="running-dot" />
      {props.count} running
    </div>
  )
}
