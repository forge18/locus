import { For, Match, Show, Switch, createMemo, createSignal } from 'solid-js'
import { SessionCard } from './SessionCard'
import { Transcript } from './Transcript'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { Resizable } from '../../panes/Resizable'
import {
  GUARDRAIL_NOTE,
  HANDOFF_SUMMARY,
  PTY_NOTE,
  SESSION_LIST_FOOTER,
  WAITING_NOTE,
  useDefaultDetailId,
  useSessionDetails,
} from '../../data/sessions'

export interface AgentsViewProps {
  /** The Kanban is the board fixture; routing stays with the shell. */
  onShowKanban?: () => void
  /** Minimize sends the session to the strip. It does not end it. */
  onMinimize?: (id: string) => void
  /**
   * Detach opens a second Tauri **window** running this same app in detached
   * mode. Never a second webview in one window — multiwebview is behind an
   * unstable flag and multi-window is ordinary.
   */
  onDetach?: (locator: string) => void
}

export function AgentsView(props: AgentsViewProps) {
  const sessions = useSessionDetails()
  const [selectedId, setSelectedId] = createSignal(useDefaultDetailId())
  const [minimized, setMinimized] = createSignal<string[]>([])

  const selected = createMemo(() => sessions.find((s) => s.id === selectedId()) ?? sessions[0])
  const running = () => sessions.filter((s) => s.status !== 'done').length

  return (
    <div class="agents" data-testid="agents">
      <div class="automate-view-switcher" data-testid="automate-view-switcher">
        <button
          type="button"
          class="automate-view-tab"
          data-testid="automate-kanban-tab"
          aria-pressed="false"
          onClick={() => props.onShowKanban?.()}
        >
          Kanban
        </button>
        <button
          type="button"
          class="automate-view-tab"
          data-testid="automate-list-tab"
          aria-pressed="true"
        >
          List
        </button>
      </div>

      <div class="agents-body">
      <Resizable width={356} min={280} max={520} side="right" class="session-list" testId="session-list">
        <div class="session-list-head" data-testid="session-list-head">
          Agents
          <span class="session-list-note" data-testid="session-list-count">
            {running()} running · one session each
          </span>
          <span style={{ 'margin-left': 'auto', display: 'flex', gap: 'var(--g-3)' }}>
            <Icon name="funnel" size={11} label="Filter" />
            <Icon name="sort-ascending" size={11} style={{ color: 'var(--ac)' }} label="Sort" />
          </span>
        </div>

        <div class="session-list-body">
          <For each={sessions}>
            {(session) => (
              <SessionCard
                session={session}
                selected={selected().id === session.id}
                onSelect={() => setSelectedId(session.id)}
              />
            )}
          </For>
        </div>

        <footer class="session-list-foot" data-testid="session-list-foot">
          {SESSION_LIST_FOOTER}
        </footer>
      </Resizable>

      <section class="transcript-pane" data-testid="transcript-pane">
        <header class="transcript-head" data-testid="transcript-head">
          <span class={`session-dot session-dot-${selected().status}`} />
          <span class="session-project">{selected().project}</span>
          <span class="session-agent">{selected().agent}</span>
          <span class="session-role">{selected().role}</span>
          <span class="session-task" style={{ 'max-width': '220px' }}>
            {selected().task}
          </span>
          <span class="session-chip" data-testid="transcript-status">
            {selected().status}
          </span>
          <span class="transcript-locator" data-testid="transcript-locator">
            locus://{selected().project}/session/{selected().id.replace(/^sd-/, '')}
          </span>
          <button
            type="button"
            class="transcript-control"
            aria-label="Detach"
            data-testid="transcript-detach"
            onClick={() =>
              props.onDetach?.(
                `locus://${selected().project}/session/${selected().id.replace(/^sd-/, '')}`,
              )
            }
          >
            <Icon name="arrows-out-simple" size={12} />
          </button>
          <button
            type="button"
            class="transcript-control"
            aria-label="Minimize"
            data-testid="transcript-minimize"
            onClick={() => {
              setMinimized([...minimized(), selected().id])
              props.onMinimize?.(selected().id)
            }}
          >
            <Icon name="minus" size={12} />
          </button>
        </header>

        <Transcript session={selected()} />

        <Switch>
          <Match when={selected().status === 'stuck'}>
            <footer class="session-footer" data-testid="session-footer-stuck">
              <div class="guardrail-card">
                <span class="guardrail-title">
                  <Icon name="warning-octagon" weight="fill" size={12} />
                  {GUARDRAIL_NOTE}
                </span>
                <span class="guardrail-summary">{HANDOFF_SUMMARY}</span>
                <div class="guardrail-actions">
                  <Button variant="primary" data-testid="guardrail-handoff">
                    Hand off to reviewer@2
                  </Button>
                  <Button variant="secondary" data-testid="guardrail-let-it-run">
                    Let it run
                  </Button>
                </div>
              </div>
            </footer>
          </Match>
          <Match when={selected().status === 'waiting'}>
            <footer class="session-footer" data-testid="session-footer-waiting">
              <div class="waiting-card">
                <Icon name="hourglass-medium" size={12} />
                <span data-testid="waiting-note">{WAITING_NOTE}</span>
              </div>
            </footer>
          </Match>
        </Switch>

        <div class="session-status-bar" data-testid="session-status-bar">
          <span data-testid="pty-note">{PTY_NOTE}</span>
          <span style={{ 'margin-left': 'auto' }} data-testid="run-id">
            {selected().runId}
          </span>
        </div>

        <Show when={minimized().length > 0}>
          <span hidden data-testid="minimized-ids">
            {minimized().join(',')}
          </span>
        </Show>
      </section>
      </div>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default AgentsView
