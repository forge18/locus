import { RunningPill } from './RunningPill'

export interface AppTitleBarProps {
  categoryLabel: string
  viewLabel: string
  running: number
  needsYou: number
}

export function AppTitleBar(props: AppTitleBarProps) {
  return (
    <div class="titlebar" data-testid="app-titlebar">
      <div class="traffic" data-testid="traffic-lights">
        <span class="traffic-close" />
        <span class="traffic-min" />
        <span class="traffic-max" />
      </div>
      <div class="wordmark" data-testid="wordmark">
        Locus
      </div>
      <div class="title-context">
        <span data-testid="title-category">{props.categoryLabel}</span>
        <span data-testid="title-view">{props.viewLabel}</span>
      </div>
      <div style={{ flex: 1 }} />
      <RunningPill running={props.running} needsYou={props.needsYou} />
    </div>
  )
}
