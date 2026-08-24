import {
  ANALYTICS_BREAKDOWN,
  ANALYTICS_RANGES,
  ANALYTICS_STATS,
  BREAKDOWN_DIMENSIONS,
  EXTENSION_KINDS,
  EXTENSION_USAGE,
  RETRIEVAL_TIERS,
  TASK_OUTCOMES,
  TELEMETRY_ACTIONS,
  TELEMETRY_FACETS,
  TELEMETRY_SESSIONS,
  TELEMETRY_VERBS,
  WORKFLOW_TIMINGS,
} from '../fixtures/analytics'
import type { AnalyticsMeasure, AnalyticsRange, AnalyticsScope, BreakdownDimension, ExtensionKind } from '../fixtures/analytics'

export * from '../fixtures/analytics'

export function resolveAnalyticsRange(range: AnalyticsRange) {
  return ANALYTICS_RANGES.find((candidate) => candidate.value === range) ?? ANALYTICS_RANGES[2]
}

/** Every projection receives this explicit scope; global analytics never reads the project selector. */
export interface AnalyticsQuery { scope: AnalyticsScope; range: AnalyticsRange }
export function queryAnalytics(query: AnalyticsQuery) {
  return { ...query, resolvedRange: resolveAnalyticsRange(query.range) }
}

export function useAnalyticsStats(_query: AnalyticsQuery = { scope: 'all', range: '30d' }) {
  return ANALYTICS_STATS
}

export function useBreakdown(_query: AnalyticsQuery, _dimension: BreakdownDimension, _measure: AnalyticsMeasure = 'spend') {
  return ANALYTICS_BREAKDOWN
}

export function useTaskOutcomes(query: AnalyticsQuery) {
  return query.scope === 'all' ? TASK_OUTCOMES.filter((outcome) => outcome.label !== 'Landed after rework') : TASK_OUTCOMES
}

export function useWorkflowTimings(_query: AnalyticsQuery) { return WORKFLOW_TIMINGS }
export function useRetrievalTiers(_query: AnalyticsQuery) { return RETRIEVAL_TIERS }
export function useExtensionUsage(_query: AnalyticsQuery, kind: ExtensionKind = 'all') {
  return kind === 'all' ? EXTENSION_USAGE : EXTENSION_USAGE.filter((extension) => extension.kind === kind)
}
export function useExtensionKinds() { return EXTENSION_KINDS }
export function useBreakdownDimensions() { return BREAKDOWN_DIMENSIONS }

export function useTelemetryFacets(_query: AnalyticsQuery, _search = '') { return TELEMETRY_FACETS }
export function useTelemetryActions(_query: AnalyticsQuery, _search = '') { return TELEMETRY_ACTIONS }
export function useTelemetrySessions(_query: AnalyticsQuery, _search = '') { return TELEMETRY_SESSIONS }
export function useTelemetryVerbs() { return TELEMETRY_VERBS }
