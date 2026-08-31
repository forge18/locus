import {
  ACTION_ROWS,
  ALL_SESSION_ROWS,
  FACETS,
  FACET_GROUPS,
  FILTER_CHIPS,
  METRICS,
  SESSION_ROWS,
  SPARKLINE,
  SPEND,
  TOOL_ROWS,
  VERB_COUNTS,
} from '../fixtures/telemetry'
import type {
  ActionRow,
  Facet2,
  FacetGroup,
  FilterChip,
  SessionRow,
  SpendRow,
  TelemetryMetric,
  ToolRow,
  VerbCount,
} from '../fixtures/telemetry'
import type { AnalyticsRange, AnalyticsScope } from '../fixtures/analytics'
import { dataProvider } from './provider'
import type { Envelope } from './envelope'

export function fetchTelemetryMetrics(
  scope: AnalyticsScope = 'all',
  range: AnalyticsRange = '30d',
): Promise<Envelope<TelemetryMetric[]>> {
  return dataProvider().query<TelemetryMetric>('telemetry_metrics', {
    query: { scope, range },
  })
}

export {
  ACTION_NOTE,
  MISSING_VERB_NOTE,
  RESET_LABEL,
  SEARCH_NOTE,
  SEARCH_QUERY,
  SESSION_TOTAL,
  TOOL_ANOMALY,
  TOOL_NOTE,
} from '../fixtures/telemetry'
export type {
  ActionRow,
  Facet,
  FacetGroup,
  FilterChip,
  SessionRow,
  SpendRow,
  TelemetryMetric,
  ToolRow,
  VerbCount,
} from '../fixtures/telemetry'

/** Becomes: invoke("telemetry_metrics") */
export function useTelemetryMetrics(): TelemetryMetric[] {
  return METRICS
}

/** Becomes: invoke("sessions_over_time") */
export function useSparkline(): number[] {
  return SPARKLINE
}

/** Becomes: invoke("telemetry_filters") */
export function useFilterChips(): FilterChip[] {
  return FILTER_CHIPS
}

/** Becomes: invoke("telemetry_facets") */
export function useFacetGroups(): FacetGroup[] {
  return FACET_GROUPS
}

/** Becomes: invoke("telemetry_actions") */
export function useActionRows(): ActionRow[] {
  return ACTION_ROWS
}

/** Becomes: invoke("telemetry_tools") */
export function useToolRows(): ToolRow[] {
  return TOOL_ROWS
}

/** How many rows a page carries. */
export const PAGE_SIZE = 100

/** Becomes: invoke("telemetry_sessions") — the first page, for the tests that read it. */
export function useSessionRows(): SessionRow[] {
  return SESSION_ROWS
}

/**
 * Becomes: invoke("telemetry_sessions_page", { offset, limit, filter })
 *
 * A page at a time, so opening Telemetry costs one page rather than 300 rows.
 */
export function useSessionRowsPage(offset: number, limit = PAGE_SIZE): SessionRow[] {
  return ALL_SESSION_ROWS.slice(offset, offset + limit)
}

/** Becomes: invoke("telemetry_sessions_count", { filter }) */
export function useSessionRowCount(): number {
  return ALL_SESSION_ROWS.length
}

/** Becomes: invoke("telemetry_verb_counts") */
export function useVerbCounts(): VerbCount[] {
  return VERB_COUNTS
}

/** Becomes: invoke("telemetry_facets", { flat: true }) */
export function useFacets(): Facet2[] {
  return FACETS
}

/** Becomes: invoke("telemetry_spend") */
export function useSpend(): SpendRow[] {
  return SPEND
}
