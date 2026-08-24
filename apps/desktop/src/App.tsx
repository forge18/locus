import { Match, Suspense, Switch, lazy, onMount } from 'solid-js'
import { Shell } from './shell/Shell'
import { EmptyPane } from './ui/EmptyPane'
import { SkeletonRows } from './ui/SkeletonRows'
import { mountIconSprite } from './ui/sprite'
import { createNavStore } from './nav'
import { applyTheme, savedTheme } from './styles/theme'
import { DesktopPlaceholder } from './screens/DesktopPlaceholder'
import './styles/app.css'

const InboxView = lazy(() => import('./screens/inbox/InboxView'))
const StatusView = lazy(() => import('./screens/status/StatusView'))
const PlanView = lazy(() => import('./screens/plan/PlanView'))
import { ProjectsView } from './screens/projects/ProjectsView'
const SessionsView = lazy(() => import('./screens/automate/AgentsView'))
const TelemetryView = lazy(() => import('./screens/review/TelemetryView'))
const RunsView = lazy(() => import('./screens/review/RunsView'))
const ArtifactView = lazy(() => import('./screens/review/ArtifactsView'))
const WikiView = lazy(() => import('./screens/wiki/WikiView'))
const AgentsView = lazy(() => import('./screens/workshop/AgentDefsView'))
const HarnessesView = lazy(() => import('./screens/workshop/HarnessesView'))

function App() {
  const nav = createNavStore()
  onMount(() => {
    applyTheme(document.documentElement, savedTheme(window.localStorage))
    mountIconSprite()
  })

  return (
    <Shell nav={nav}>
      <Suspense fallback={<SkeletonRows count={8} rowHeight={26} />}>
        <Switch fallback={<EmptyPane icon="signpost" reason={`No screen is built for ${nav.view()} yet.`} />}>
          <Match when={nav.view() === 'inbox'}><InboxView nav={nav} /></Match>
          <Match when={nav.view() === 'status'}><StatusView /></Match>
          <Match when={nav.view() === 'telemetry'}><TelemetryView /></Match>
          <Match when={nav.view() === 'projects'}><ProjectsView /></Match>
          <Match when={nav.view() === 'plan'}><PlanView /></Match>
          <Match when={nav.view() === 'sessions'}><SessionsView onShowKanban={() => nav.go('sessions')} /></Match>
          <Match when={nav.view() === 'runs'}><RunsView /></Match>
          <Match when={nav.view() === 'artifact'}><ArtifactView artifactId={nav.params().artifactId} /></Match>
          <Match when={nav.view() === 'wiki'}><WikiView nav={nav} /></Match>
          <Match when={nav.view() === 'agents'}><AgentsView onNavigate={nav.go} /></Match>
          <Match when={nav.view() === 'harnesses'}><HarnessesView /></Match>
          <Match when={nav.view() === 'mail'}><DesktopPlaceholder view="mail" /></Match>
          <Match when={nav.view() === 'interact'}><DesktopPlaceholder view="interact" /></Match>
          <Match when={nav.view() === 'qa'}><DesktopPlaceholder view="qa" /></Match>
          <Match when={nav.view() === 'autorun'}><DesktopPlaceholder view="autorun" /></Match>
          <Match when={nav.view() === 'schedule'}><DesktopPlaceholder view="schedule" /></Match>
          <Match when={nav.view() === 'short'}><DesktopPlaceholder view="short" /></Match>
          <Match when={nav.view() === 'memory'}><DesktopPlaceholder view="memory" /></Match>
          <Match when={nav.view() === 'settings'}><DesktopPlaceholder view="settings" /></Match>
          <Match when={nav.view() === 'cli'}><DesktopPlaceholder view="cli" /></Match>
          <Match when={nav.view() === 'commands'}><DesktopPlaceholder view="commands" /></Match>
          <Match when={nav.view() === 'hooks'}><DesktopPlaceholder view="hooks" /></Match>
          <Match when={nav.view() === 'linters'}><DesktopPlaceholder view="linters" /></Match>
          <Match when={nav.view() === 'styles'}><DesktopPlaceholder view="styles" /></Match>
          <Match when={nav.view() === 'providers'}><DesktopPlaceholder view="providers" /></Match>
          <Match when={nav.view() === 'rules'}><DesktopPlaceholder view="rules" /></Match>
          <Match when={nav.view() === 'skills'}><DesktopPlaceholder view="skills" /></Match>
          <Match when={nav.view() === 'canvas'}><DesktopPlaceholder view="canvas" /></Match>
          <Match when={nav.view() === 'workflows'}><DesktopPlaceholder view="workflows" /></Match>
        </Switch>
      </Suspense>
    </Shell>
  )
}

export default App
