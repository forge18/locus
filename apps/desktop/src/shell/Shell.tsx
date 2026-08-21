import { createSignal, onCleanup, onMount } from 'solid-js'
import type { JSX } from 'solid-js'
import { Rail } from './Rail'
import { Strip } from './Strip'
import { TabBar } from './TabBar'
import { TitleBar } from './TitleBar'
import { LocatorPalette } from '../nav/LocatorPalette'
import { Sheet } from '../ui/Sheet'
import { useProjects } from '../data/core'
import { useInboxItems } from '../data/inbox'
import { useRunningCount, useStripCards } from '../data/strip'
import type { NavStore } from '../nav'

export interface ShellProps {
  nav: NavStore
  children: JSX.Element
}

/**
 * The four bands, composed once. A screen renders into the body between the tab
 * bar and the strip and never draws any of this itself.
 */
export function Shell(props: ShellProps) {
  const [selectedProjects, setSelectedProjects] = createSignal<string[]>([])
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  const projects = useProjects().map((p) => ({ id: p.id, name: p.name }))

  // ⌘K resolves a locator. It is bound here because the locator bar it opens is
  // shell chrome, and there is one of it per window.
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
      <TitleBar
        locatorPath={props.nav.locatorPath()}
        projects={projects}
        selectedProjects={selectedProjects()}
        onProjectsChange={setSelectedProjects}
        runningCount={useRunningCount()}
        onOpenLocator={() => setPaletteOpen(true)}
      />
      <div class="body">
        <Rail
          view={props.nav.view()}
          onNavigate={props.nav.go}
          inboxCount={useInboxItems().length}
        />
        <div class="main">
          <TabBar
            view={props.nav.view()}
            onNavigate={props.nav.go}
            locator={props.nav.locatorPath()}
          />
          <div class="screen" data-testid="screen">
            {props.children}
          </div>
        </div>
      </div>
      <Strip cards={useStripCards()} />

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
