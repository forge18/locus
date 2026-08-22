import { Match, Suspense, Switch, lazy, onMount } from 'solid-js'
import { Shell } from './shell/Shell'
import { EmptyPane } from './ui/EmptyPane'
import { SkeletonRows } from './ui/SkeletonRows'
import { mountIconSprite } from './ui/sprite'
import { createNavStore } from './nav'

import './styles/app.css'

/**
 * Screens load on the view that needs them.
 *
 * Fourteen screens in one bundle means every one of them is parsed before the
 * inbox paints, and most sessions never open Workshop at all. Splitting at the
 * view boundary is free: the nav store already decides which one is next.
 */
const InboxView = lazy(() => import('./screens/inbox/InboxView'))
const StatusView = lazy(() => import('./screens/status/StatusView'))
const PlanView = lazy(() => import('./screens/plan/PlanView'))
const WikiView = lazy(() => import('./screens/wiki/WikiView'))
const DevelopView = lazy(() => import('./screens/develop/DevelopView'))
const KanbanView = lazy(() => import('./screens/automate/KanbanView'))
const AgentsView = lazy(() => import('./screens/automate/AgentsView'))
const TelemetryView = lazy(() => import('./screens/review/TelemetryView'))
const RunsView = lazy(() => import('./screens/review/RunsView'))
const ArtifactsView = lazy(() => import('./screens/review/ArtifactsView'))
const ExtensionsView = lazy(() => import('./screens/workshop/ExtensionsView'))
const AgentDefsView = lazy(() => import('./screens/workshop/AgentDefsView'))
const WorkflowView = lazy(() => import('./screens/workshop/WorkflowView'))
const HarnessesView = lazy(() => import('./screens/workshop/HarnessesView'))

function App() {
  const nav = createNavStore()
  onMount(() => mountIconSprite())

  return (
    <Shell nav={nav}>
      {/* Skeleton rows at a real height, so the screen does not reflow when it lands. */}
      <Suspense fallback={<SkeletonRows count={8} rowHeight={26} />}>
        <Switch
          fallback={
            <EmptyPane
              icon="signpost"
              reason={`No screen is built for ${nav.view()} yet — the shell is, and this is where the screen will render.`}
            />
          }
        >
          <Match when={nav.view() === 'inbox'}>
            <InboxView nav={nav} />
          </Match>
          <Match when={nav.view() === 'status'}>
            <StatusView />
          </Match>
          <Match when={nav.view() === 'plan'}>
            <PlanView />
          </Match>
          <Match when={nav.view() === 'wiki'}>
            <WikiView nav={nav} />
          </Match>
          <Match when={nav.view() === 'develop'}>
            <DevelopView />
          </Match>
          <Match when={nav.view() === 'board'}>
            <KanbanView onShowAgents={() => nav.go('sessions')} />
          </Match>
          <Match when={nav.view() === 'sessions'}>
            <AgentsView onShowKanban={() => nav.go('board')} />
          </Match>
          <Match when={nav.view() === 'telemetry'}>
            <TelemetryView />
          </Match>
          <Match when={nav.view() === 'runs'}>
            <RunsView />
          </Match>
          <Match when={nav.view() === 'artifact'}>
            <ArtifactsView artifactId={nav.params().artifactId} />
          </Match>
          <Match when={nav.view() === 'extensions'}>
            <ExtensionsView onNavigate={nav.go} />
          </Match>
          <Match when={nav.view() === 'agents'}>
            <AgentDefsView onNavigate={nav.go} />
          </Match>
          <Match when={nav.view() === 'canvas'}>
            <WorkflowView />
          </Match>
          <Match when={nav.view() === 'harnesses'}>
            <HarnessesView />
          </Match>
        </Switch>
      </Suspense>
    </Shell>
  )
}

export default App
