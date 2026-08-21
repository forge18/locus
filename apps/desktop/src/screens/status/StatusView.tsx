import { For } from 'solid-js'
import { MetricCard } from './MetricCard'
import { RunsByHour } from './RunsByHour'
import { WantsAttention } from './WantsAttention'
import { Table } from '../../ui/Table'
import type { Column } from '../../ui/Table'
import {
  useProjectRows,
  useRunsByHour,
  useStatusMetrics,
  useWantsAttention,
} from '../../data/status'
import type { ProjectRow } from '../../data/status'

/** Unknown is not zero, and it says so rather than showing a number nobody measured. */
const orUnknown = (value: string | null) =>
  value === null ? <span class="unknown">unknown</span> : value

const COLUMNS: Column<ProjectRow>[] = [
  { key: 'project', header: 'Project', cell: (r) => r.project },
  { key: 'repos', header: 'Repos', type: 'numeric', cell: (r) => r.repos },
  { key: 'running', header: 'Running', type: 'numeric', cell: (r) => r.running },
  { key: 'inReview', header: 'In review', type: 'numeric', cell: (r) => r.inReview },
  {
    key: 'verify',
    header: 'Verify',
    type: 'numeric',
    // Below half means the loop is failing more than it passes, which is the
    // threshold the design colours on.
    cell: (r) => <span class={r.verify >= 50 ? 'verify-ok' : 'verify-bad'}>{r.verify}%</span>,
  },
  { key: 'tokens', header: 'Tokens today', type: 'numeric', cell: (r) => orUnknown(r.tokensToday) },
  { key: 'cache', header: 'Cache', type: 'numeric', cell: (r) => orUnknown(r.cache) },
  { key: 'lastEvent', header: 'Last event', cell: (r) => r.lastEvent },
]

/**
 * What I need to know, now. Digging into a run that went wrong is Review's job:
 * Status carries no search, no filter chips and no facets, and keeping it that way
 * is what stops it becoming a second Review.
 */
export function StatusView() {
  return (
    <div class="status" data-testid="status">
      <div class="status-metrics" data-testid="status-metrics">
        <For each={useStatusMetrics()}>{(metric) => <MetricCard metric={metric} />}</For>
      </div>

      <div class="status-middle">
        <RunsByHour hours={useRunsByHour()} />
        <WantsAttention rows={useWantsAttention()} />
      </div>

      <section class="panel" data-testid="project-table">
        <span class="panel-title">Projects</span>
        <Table columns={COLUMNS} rows={useProjectRows()} rowKey={(r) => r.project} />
      </section>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default StatusView
