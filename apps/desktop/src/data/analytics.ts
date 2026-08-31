import {
  ANALYTICS_RANGES,
  TELEMETRY_ACTIONS,
  TELEMETRY_FACETS,
  TELEMETRY_SESSIONS,
  TELEMETRY_VERBS,
} from "./demo/fixtures/analytics";
import type {
  AnalyticsMeasure,
  AnalyticsRange,
  AtAGlanceMetric,
  AnalyticsScope,
  BreakdownDimension,
  ExtensionKind,
  AnalyticsStat,
  BreakdownRow,
  TaskOutcome,
  WorkflowTiming,
  RetrievalTier,
  ExtensionUsage,
} from "./demo/fixtures/analytics";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export * from "./demo/fixtures/analytics";

/** Live scoped activity metrics used by the Analytics surface. */
export function fetchAtAGlanceMetrics(
  scope: AnalyticsScope = "all",
  range: AnalyticsRange = "30d",
): Promise<Envelope<AtAGlanceMetric[]>> {
  return dataProvider().query<AtAGlanceMetric>("analytics_at_a_glance", {
    query: { scope, range },
  });
}

/** Becomes: invoke('analytics_at_a_glance', { query }) */
export function useAtAGlanceMetrics(
  query: AnalyticsQuery = { scope: "all", range: "30d" },
) {
  return (
    dataProvider().read?.<AtAGlanceMetric[]>("analytics_at_a_glance", {
      query,
    }) ?? []
  );
}

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
  query: AnalyticsQuery = { scope: "all", range: "30d" },
) {
  return (
    dataProvider().read?.<AnalyticsStat[]>("analytics_stats", { query }) ?? []
  );
}

/** Becomes: invoke('analytics_breakdown', { query, dimension, measure }) */
export function useBreakdown(
  query: AnalyticsQuery,
  dimension: BreakdownDimension,
  measure: AnalyticsMeasure = "spend",
) {
  return (
    dataProvider().read?.<BreakdownRow[]>("analytics_breakdown", {
      query,
      dimension,
      measure,
    }) ?? []
  );
}

/** Becomes: invoke('analytics_task_outcomes', { query }) */
export function useTaskOutcomes(query: AnalyticsQuery) {
  const outcomes =
    dataProvider().read?.<TaskOutcome[]>("analytics_task_outcomes", {
      query,
    }) ?? [];
  return query.scope === "all"
    ? outcomes.filter((outcome) => outcome.label !== "Landed after rework")
    : outcomes;
}

/** Becomes: invoke('analytics_workflow_timings', { query }) */
export function useWorkflowTimings(query: AnalyticsQuery) {
  return (
    dataProvider().read?.<WorkflowTiming[]>("analytics_workflow_timings", {
      query,
    }) ?? []
  );
}
/** Becomes: invoke('analytics_retrieval_tiers', { query }) */
export function useRetrievalTiers(query: AnalyticsQuery) {
  return (
    dataProvider().read?.<RetrievalTier[]>("analytics_retrieval_tiers", {
      query,
    }) ?? []
  );
}
/** Becomes: invoke('analytics_extension_usage', { query, kind }) */
export function useExtensionUsage(
  query: AnalyticsQuery,
  kind: ExtensionKind = "all",
) {
  const usage =
    dataProvider().read?.<ExtensionUsage[]>("analytics_extension_usage", {
      query,
      kind,
    }) ?? [];
  return kind === "all"
    ? usage
    : usage.filter((extension) => extension.kind === kind);
}
/** Becomes: invoke('analytics_extension_kinds') */
export function useExtensionKinds() {
  return (
    dataProvider().read?.<ExtensionKind[]>("analytics_extension_kinds") ?? []
  );
}
/** Becomes: invoke('analytics_breakdown_dimensions') */
export function useBreakdownDimensions() {
  return (
    dataProvider().read?.<BreakdownDimension[]>(
      "analytics_breakdown_dimensions",
    ) ?? []
  );
}

/** Becomes: invoke('analytics_telemetry_facets', { query, search }) */
export function useTelemetryFacets(query: AnalyticsQuery, search = "") {
  return (
    dataProvider().read?.<typeof TELEMETRY_FACETS>(
      "analytics_telemetry_facets",
      {
        query,
        search,
      },
    ) ?? []
  );
}
/** Becomes: invoke('analytics_telemetry_actions', { query, search }) */
export function useTelemetryActions(query: AnalyticsQuery, search = "") {
  return (
    dataProvider().read?.<typeof TELEMETRY_ACTIONS>(
      "analytics_telemetry_actions",
      {
        query,
        search,
      },
    ) ?? []
  );
}
/** Becomes: invoke('analytics_telemetry_sessions', { query, search }) */
export function useTelemetrySessions(query: AnalyticsQuery, search = "") {
  return (
    dataProvider().read?.<typeof TELEMETRY_SESSIONS>(
      "analytics_telemetry_sessions",
      {
        query,
        search,
      },
    ) ?? []
  );
}
/** Becomes: invoke('analytics_telemetry_verbs') */
export function useTelemetryVerbs() {
  return (
    dataProvider().read?.<typeof TELEMETRY_VERBS>(
      "analytics_telemetry_verbs",
    ) ?? []
  );
}
