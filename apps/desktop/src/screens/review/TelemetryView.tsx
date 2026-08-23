import { For, Show, createSignal } from 'solid-js'
import { Icon } from '../../ui/Icon'
import { VirtualTable } from '../../panes/VirtualTable'
import type { Column } from '../../ui/Table'
import {
  ACTION_NOTE,
  MISSING_VERB_NOTE,
  RESET_LABEL,
  SEARCH_NOTE,
  SEARCH_QUERY,
  PAGE_SIZE,
  SESSION_TOTAL,
  TOOL_ANOMALY,
  TOOL_NOTE,
  useActionRows,
  useFacetGroups,
  useFilterChips,
  useSessionRowCount,
  useSessionRowsPage,
  useSparkline,
  useTelemetryMetrics,
  useToolRows,
} from '../../data/telemetry'
import type { SessionRow } from '../../data/telemetry'

const orUnknown = (value: string | null) =>
  value === null ? <span class="unknown">unknown</span> : value

const COLUMNS: Column<SessionRow>[] = [
  { key: 'when', header: 'When ↓', type: 'mono', cell: (r) => r.when },
  { key: 'harness', header: 'Harness', cell: (r) => r.harness },
  { key: 'project', header: 'Project · repo', cell: (r) => `${r.project} · ${r.repo}` },
  { key: 'agent', header: 'Agent · role', type: 'mono', cell: (r) => `${r.agent} · ${r.role}` },
  { key: 'models', header: 'Model(s)', type: 'mono', cell: (r) => r.models },
  { key: 'runs', header: 'Runs', type: 'numeric', cell: (r) => r.runs },
  { key: 'events', header: 'Events', type: 'numeric', cell: (r) => r.events.toLocaleString('en-US') },
  {
    key: 'errors',
    header: 'Errors',
    type: 'numeric',
    cell: (r) => <span class={r.errors > 0 ? 'verify-bad' : ''}>{r.errors}</span>,
  },
  { key: 'tokens', header: 'Tokens', type: 'numeric', cell: (r) => orUnknown(r.tokens) },
  {
    key: 'status',
    header: 'Status',
    cell: (r) => (
      <span class={`status-${r.status.replace(/\s+/g, '-')}`}>
        {r.statusDetail ? `${r.status} ${r.statusDetail}` : r.status}
      </span>
    ),
  },
  { key: 'id', header: 'Id', type: 'mono', cell: (r) => r.id },
]

/**
 * Review is after; Dashboard is now. Every number here is already a column, so
 * this screen is a query rather than new instrumentation.
 */
/** Rows are 26px; the table body is drawn at 300px. */
const ROW_HEIGHT = 26
const BODY_HEIGHT = 300

export function TelemetryView() {
  const sessionTotal = useSessionRowCount()
  const [loaded, setLoaded] = createSignal(useSessionRowsPage(0))

  /** One page at a time, as the window approaches the end of what is loaded. */
  const loadMore = () => {
    if (loaded().length >= sessionTotal) return
    setLoaded([...loaded(), ...useSessionRowsPage(loaded().length, PAGE_SIZE)])
  }

  const actions = useActionRows()
  const tools = useToolRows()
  const maxAction = Math.max(...actions.map((a) => a.count))
  const maxTool = Math.max(...tools.map((t) => t.count))

  return (
    <div class="telemetry" data-testid="telemetry" data-desktop-route="review-telemetry" data-filter-evidence="available">
      <div class="tm-search" data-testid="tm-search">
        <Icon name="magnifying-glass" size={12} style={{ color: 'var(--text-muted)' }} />
        <span class="tm-query" data-testid="tm-query">
          {SEARCH_QUERY}
        </span>
        <span class="tm-caret blink" data-testid="tm-caret" />
        <span class="tm-search-note" data-testid="tm-search-note">
          {SEARCH_NOTE}
        </span>
      </div>

      <div class="tm-chips" data-testid="tm-chips">
        <For each={useFilterChips()}>
          {(chip) => (
            <span
              class="tag tag-outline"
              data-testid={`tm-chip-${chip.label.replace(/\W+/g, '-')}`}
              data-active={chip.active ? 'true' : undefined}
            >
              {chip.label}
            </span>
          )}
        </For>
        <button type="button" class="tm-reset" data-testid="tm-reset">
          {RESET_LABEL}
        </button>
      </div>

      <div class="tm-metrics" data-testid="tm-metrics">
        <For each={useTelemetryMetrics()}>
          {(metric) => (
            <div
              class={['metric-card', metric.bad ? 'tm-metric-bad' : ''].filter(Boolean).join(' ')}
              data-testid={`tm-metric-${metric.label.toLowerCase().replace(/\s+/g, '-')}`}
              data-bad={metric.bad ? 'true' : undefined}
            >
              <span class="metric-label">{metric.label}</span>
              <div class="metric-value">
                <span class="metric-numeral">{metric.value}</span>
                <Show when={metric.unit}>
                  <span class="metric-unit">{metric.unit}</span>
                </Show>
              </div>
            </div>
          )}
        </For>
        <div class="metric-card" data-testid="tm-sparkline-card">
          <span class="metric-label">Sessions over time</span>
          <div class="sparkline" data-testid="tm-sparkline">
            <For each={useSparkline()}>
              {(value) => (
                <span class="sparkline-bar" style={{ height: `${value}%` }} />
              )}
            </For>
          </div>
        </div>
      </div>

      <div class="tm-band" data-testid="tm-band">
        <section class="tm-panel" data-testid="tm-filters">
          <div class="tm-panel-head">
            <span class="panel-title">Filters</span>
          </div>
          <For each={useFacetGroups()}>
            {(group) => (
              <div class="facet-group" data-testid={`facet-group-${group.key}`}>
                <span class="facet-group-label">{group.label}</span>
                <div class="facet-chips">
                  <For each={group.facets}>
                    {(facet) => (
                      <button
                        type="button"
                        class={['facet', facet.invariant ? 'facet-invariant' : '']
                          .filter(Boolean)
                          .join(' ')}
                        data-testid={`facet-${group.key}-${facet.value.replace(/\W+/g, '-')}`}
                        aria-pressed={facet.active ? 'true' : 'false'}
                      >
                        {facet.value}
                        <span class="facet-count">{facet.count}</span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>
        </section>

        <section class="tm-panel" data-testid="tm-actions">
          <div class="tm-panel-head">
            <span class="panel-title">Actions</span>
            <span class="tm-panel-note">{ACTION_NOTE}</span>
          </div>
          <For each={actions}>
            {(action) => (
              <Show
                when={!action.alarm}
                fallback={
                  <>
                    <div
                      class="bar-row bar-row-bad"
                      data-testid={`action-${action.verb}`}
                      data-alarm="true"
                    >
                      <span class="bar-label">{action.verb}</span>
                      <span class="bar-track">
                        <span
                          class="bar-fill bar-fill-bad"
                          style={{ width: `${(action.count / maxAction) * 100}%` }}
                        />
                      </span>
                      <span class="bar-count">{action.count.toLocaleString('en-US')}</span>
                    </div>
                    <div class="alarm-callout" data-testid="permission-alarm">
                      <Icon name="warning-octagon" weight="fill" size={12} style={{ 'flex-shrink': 0, color: 'var(--status-danger)' }} />
                      <span>{action.alarm}</span>
                    </div>
                  </>
                }
              >
                <div
                  class={['bar-row', action.bad ? 'bar-row-bad' : ''].filter(Boolean).join(' ')}
                  data-testid={`action-${action.verb}`}
                >
                  <span class="bar-label">{action.verb}</span>
                  <span class="bar-track">
                    <span
                      class={['bar-fill', action.bad ? 'bar-fill-bad' : ''].filter(Boolean).join(' ')}
                      style={{ width: `${(action.count / maxAction) * 100}%` }}
                    />
                  </span>
                  <span class="bar-count">{action.count.toLocaleString('en-US')}</span>
                </div>
              </Show>
            )}
          </For>
          <span class="tm-footnote" data-testid="missing-verb-note">
            {MISSING_VERB_NOTE}
          </span>
        </section>

        <section class="tm-panel" data-testid="tm-tools">
          <div class="tm-panel-head">
            <span class="panel-title">Tools</span>
            <span class="tm-panel-note">{TOOL_NOTE}</span>
          </div>
          <For each={tools}>
            {(tool) => (
              <div class="bar-row" data-testid={`tool-${tool.tool.replace(/\W+/g, '-')}`}>
                <span class="bar-label bar-label-tool">{tool.tool}</span>
                <span class="bar-track">
                  <span class="bar-fill" style={{ width: `${(tool.count / maxTool) * 100}%` }} />
                </span>
                <span class="bar-count">{tool.count.toLocaleString('en-US')}</span>
              </div>
            )}
          </For>
          <span class="tm-footnote" data-testid="tool-anomaly">
            {TOOL_ANOMALY}
          </span>
        </section>
      </div>

      <section class="panel" data-testid="tm-sessions">
        <span class="panel-title">Sessions ({SESSION_TOTAL})</span>
        <VirtualTable
          testId="tm-sessions-table"
          columns={COLUMNS}
          rows={loaded()}
          total={sessionTotal}
          rowKey={(r) => r.id}
          rowHeight={ROW_HEIGHT}
          height={BODY_HEIGHT}
          onLoadMore={loadMore}
        />
      </section>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default TelemetryView
