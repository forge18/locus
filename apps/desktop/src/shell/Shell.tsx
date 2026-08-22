import { createSignal, onCleanup, onMount } from 'solid-js'
import type { JSX } from 'solid-js'
import { AppTitleBar } from './AppTitleBar'
import { ProjectRail } from './ProjectRail'
import { LocatorPalette } from '../nav/LocatorPalette'
import { Sheet } from '../ui/Sheet'
import { useRunningCount, useStripCards } from '../data/strip'
import type { ActiveSession } from './RunningPill'
import type { NavStore } from '../nav'

export interface ShellProps {
  nav: NavStore
  children: JSX.Element
}

/** The v2 title bar and project-scoped rail frame every screen. */
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
        <ProjectRail selectedProject={props.nav.params().project} />
        <div class="main">
          <div class="screen" data-testid="screen">
            {props.children}
          </div>
        </div>
      </div>

      <LocatorPalette
        open={paletteOpen()}
        onOpenChange={setPaletteOpen}
        current={props.nav.locator()}
        onResolve={props.nav.open}
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
