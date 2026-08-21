import { LocatorBar } from './LocatorBar'
import { ProjectFilter } from './ProjectFilter'
import { RunningCount } from './RunningCount'

export interface TitleBarProps {
  locatorPath: string
  projects: { id: string; name: string }[]
  selectedProjects: string[]
  onProjectsChange: (selected: string[]) => void
  runningCount: number
  onOpenLocator?: () => void
}

export function TitleBar(props: TitleBarProps) {
  return (
    <div class="titlebar" data-testid="titlebar">
      {/* Drawn for macOS. What this becomes on Windows and Linux is undecided. */}
      <div class="traffic" data-testid="traffic-lights">
        <span class="traffic-close" />
        <span class="traffic-min" />
        <span class="traffic-max" />
      </div>
      <div class="wordmark" data-testid="wordmark">
        Locus
      </div>
      <div style={{ flex: 1, display: 'flex', 'justify-content': 'center' }}>
        <LocatorBar path={props.locatorPath} onOpen={props.onOpenLocator} />
      </div>
      <ProjectFilter
        projects={props.projects}
        selected={props.selectedProjects}
        onChange={props.onProjectsChange}
      />
      <RunningCount count={props.runningCount} />
    </div>
  )
}
