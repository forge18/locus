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
} from "../fixtures/analytics";
import type {
  AnalyticsMeasure,
  AnalyticsRange,
  AnalyticsScope,
  BreakdownDimension,
  ExtensionKind,
} from "../fixtures/analytics";

export * from "../fixtures/analytics";

export function resolveAnalyticsRange(range: AnalyticsRange) {
  return (
    ANALYTICS_RANGES.find((candidate) => candidate.value === range) ??
    ANALYTICS_RANGES[2]
  );
}

/** Every projection receives this explicit scope; global analytics never reads the project selector. */
export interface AnalyticsQuery {
  scope: AnalyticsScope;
  range: AnalyticsRange;
}
export function queryAnalytics(query: AnalyticsQuery) {
  return { ...query, resolvedRange: resolveAnalyticsRange(query.range) };
}

/** Becomes: invoke('analytics_stats', { query }) */
export function useAnalyticsStats(
  _query: AnalyticsQuery = { scope: "all", range: "30d" },
) {
  return ANALYTICS_STATS;
}

/** Becomes: invoke('analytics_breakdown', { query, dimension, measure }) */
export function useBreakdown(
  _query: AnalyticsQuery,
  _dimension: BreakdownDimension,
  _measure: AnalyticsMeasure = "spend",
) {
  return ANALYTICS_BREAKDOWN;
}

/** Becomes: invoke('analytics_task_outcomes', { query }) */
export function useTaskOutcomes(query: AnalyticsQuery) {
  return query.scope === "all"
    ? TASK_OUTCOMES.filter((outcome) => outcome.label !== "Landed after rework")
    : TASK_OUTCOMES;
}

/** Becomes: invoke('analytics_workflow_timings', { query }) */
export function useWorkflowTimings(_query: AnalyticsQuery) {
  return WORKFLOW_TIMINGS;
}
/** Becomes: invoke('analytics_retrieval_tiers', { query }) */
export function useRetrievalTiers(_query: AnalyticsQuery) {
  return RETRIEVAL_TIERS;
}
/** Becomes: invoke('analytics_extension_usage', { query, kind }) */
export function useExtensionUsage(
  _query: AnalyticsQuery,
  kind: ExtensionKind = "all",
) {
  return kind === "all"
    ? EXTENSION_USAGE
    : EXTENSION_USAGE.filter((extension) => extension.kind === kind);
}
/** Becomes: invoke('analytics_extension_kinds') */
export function useExtensionKinds() {
  return EXTENSION_KINDS;
}
/** Becomes: invoke('analytics_breakdown_dimensions') */
export function useBreakdownDimensions() {
  return BREAKDOWN_DIMENSIONS;
}

/** Becomes: invoke('analytics_telemetry_facets', { query, search }) */
export function useTelemetryFacets(_query: AnalyticsQuery, _search = "") {
  return TELEMETRY_FACETS;
}
/** Becomes: invoke('analytics_telemetry_actions', { query, search }) */
export function useTelemetryActions(_query: AnalyticsQuery, _search = "") {
  return TELEMETRY_ACTIONS;
}
/** Becomes: invoke('analytics_telemetry_sessions', { query, search }) */
export function useTelemetrySessions(_query: AnalyticsQuery, _search = "") {
  return TELEMETRY_SESSIONS;
}
/** Becomes: invoke('analytics_telemetry_verbs') */
export function useTelemetryVerbs() {
  return TELEMETRY_VERBS;
}
