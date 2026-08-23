import { createSignal, onCleanup, onMount } from 'solid-js'
import type { JSX } from 'solid-js'
import { AppTitleBar } from './AppTitleBar'
import { ProjectRail } from './ProjectRail'
import { LocatorPalette } from '../nav/LocatorPalette'
import { Sheet } from '../ui/Sheet'
import { useRunningCount, useStripCards } from '../data/strip'
import type { ActiveSession } from './RunningPill'
import type { NavStore, View } from '../nav'
import { destinationDesktop, navigateDesktop } from '../nav/desktop-navigation'
import type { DesktopNavTarget, DesktopRouteId } from '../nav/desktop-locator'
import { Desktop_PROJECT_ROUTE_KINDS } from '../nav/desktop-route-kinds'

export interface ShellProps {
  nav: NavStore
  children: JSX.Element
}

const desktopViews: Record<DesktopRouteId, View> = {
  inbox: 'inbox',
  dashboard: 'status',
  'project-settings': 'extensions',
  'project-analytics': 'status',
  'plan-conversation': 'plan',
  'plan-spec': 'plan',
  'plan-tasks': 'plan',
  develop: 'develop',
  'automate-kanban': 'board',
  'automate-agents': 'sessions',
  'dispatch-autorun': 'runs',
  'dispatch-schedules': 'runs',
  'dispatch-runs': 'runs',
  'memory-short-term': 'wiki',
  'memory-long-term': 'wiki',
  'memory-artifacts': 'artifact',
  'memory-wiki': 'wiki',
  'review-telemetry': 'telemetry',
  'settings-guardrails': 'extensions',
  'workshop-agents': 'agents',
  'workshop-cli': 'extensions',
  'workshop-commands': 'extensions',
  'workshop-harnesses': 'harnesses',
  'workshop-hooks': 'extensions',
  'workshop-linters': 'extensions',
  'workshop-output-styles': 'extensions',
  'workshop-providers': 'extensions',
  'workshop-rules': 'extensions',
  'workshop-skills': 'extensions',
  'workflows-visual': 'canvas',
  'workflows-governance': 'canvas',
}

const desktopRoutes: Record<View, DesktopRouteId> = {
  inbox: 'inbox', status: 'dashboard', plan: 'plan-conversation', develop: 'develop',
  board: 'automate-kanban', sessions: 'automate-agents', telemetry: 'review-telemetry',
  runs: 'dispatch-runs', artifact: 'memory-artifacts', wiki: 'memory-wiki',
  extensions: 'workshop-commands', agents: 'workshop-agents',
  harnesses: 'workshop-harnesses', canvas: 'workflows-visual',
}

/** Maps every registered desktop route to the currently delivered shared surface. */
export function desktopViewFor(target: DesktopNavTarget): View {
  return desktopViews[target.route]
}

/** The desktop title bar and project-scoped rail frame every screen. */
export function Shell(props: ShellProps) {
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  const activeSessions: ActiveSession[] = useStripCards()
    .filter((card) => card.kind === 'agent')
    .map((card) => ({
      id: card.id,
      label: `${card.project} · ${card.agent}`,
      needsAttention: card.status === 'waiting' || card.status === 'stuck',
      lastActivityAt: -card.idleMinutes,
    }))
  const needsYou = activeSessions.filter((session) => session.needsAttention).length
  const openDesktopTarget = (target: DesktopNavTarget) => {
    const params = target.scope.kind === 'project'
      ? { project: target.scope.project }
      : { project: undefined }
    props.nav.go(desktopViewFor(target), params)
  }
  const openDesktopLocator = (locator: string) => openDesktopTarget(navigateDesktop(locator))
  const currentDesktopLocator = () => {
    const route = desktopRoutes[props.nav.view()]
    return Desktop_PROJECT_ROUTE_KINDS.some((candidate) => candidate.id === route)
      ? destinationDesktop(route, props.nav.params().project ?? 'tapestry')
      : destinationDesktop(route)
  }

  // ⌘K resolves a locator. It is bound here because the palette is shell
  // chrome, and there is one of it per window.
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key.toLowerCase() === 'k' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault()
      setPaletteOpen(true)
    }
  }
  onMount(() => document.addEventListener('keydown', onKeyDown))
  onCleanup(() => document.removeEventListener('keydown', onKeyDown))

  return (
    <div class="window" data-testid="window">
      <AppTitleBar
        categoryLabel={props.nav.categoryLabel()}
        viewLabel={props.nav.view()}
        running={useRunningCount()}
        needsYou={needsYou}
        sessions={activeSessions}
      />
      <div class="body">
        <ProjectRail selectedProject={props.nav.params().project ?? 'tapestry'} onNavigate={openDesktopLocator} />
        <div class="main">
          <div class="screen" data-testid="screen">
            {props.children}
          </div>
        </div>
      </div>

      <LocatorPalette
        open={paletteOpen()}
        onOpenChange={setPaletteOpen}
        current={currentDesktopLocator()}
        onResolve={openDesktopTarget}
      />
      <Sheet
        open={props.nav.detail() !== null}
        onOpenChange={(open) => !open && props.nav.closeDetail()}
        title={props.nav.detail() ? props.nav.detail()!.view : ''}
      >
        <p class="t-body" data-testid="detail-body">
          {props.nav.detail() ? JSON.stringify(props.nav.detail()!.params) : ''}
        </p>
      </Sheet>
    </div>
  )
}
