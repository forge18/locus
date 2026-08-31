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
} from "./demo/fixtures/telemetry";
import type { AnalyticsRange, AnalyticsScope } from "./demo/fixtures/analytics";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export function fetchTelemetryMetrics(
 scope: AnalyticsScope = "all",
 range: AnalyticsRange = "30d",
): Promise<Envelope<TelemetryMetric[]>> {
 return dataProvider().query<TelemetryMetric>("telemetry_metrics", {
  query: { scope, range },
 });
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
} from "./demo/fixtures/telemetry";
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
} from "./demo/fixtures/telemetry";

/** Becomes: invoke("telemetry_metrics") */
export function useTelemetryMetrics(): TelemetryMetric[] {
 return dataProvider().read?.<TelemetryMetric[]>("telemetry_metrics") ?? [];
}

/** Becomes: invoke("sessions_over_time") */
export function useSparkline(): number[] {
 return dataProvider().read?.<number[]>("sessions_over_time") ?? [];
}

/** Becomes: invoke("telemetry_filters") */
export function useFilterChips(): FilterChip[] {
 return dataProvider().read?.<FilterChip[]>("telemetry_filters") ?? [];
}

/** Becomes: invoke("telemetry_facets") */
export function useFacetGroups(): FacetGroup[] {
 return dataProvider().read?.<FacetGroup[]>("telemetry_facets") ?? [];
}

/** Becomes: invoke("telemetry_actions") */
export function useActionRows(): ActionRow[] {
 return dataProvider().read?.<ActionRow[]>("telemetry_actions") ?? [];
}

/** Becomes: invoke("telemetry_tools") */
export function useToolRows(): ToolRow[] {
 return dataProvider().read?.<ToolRow[]>("telemetry_tools") ?? [];
}

/** How many rows a page carries. */
export const PAGE_SIZE = 100;

/** Becomes: invoke("telemetry_sessions") — the first page, for the tests that read it. */
export function useSessionRows(): SessionRow[] {
 return dataProvider().read?.<SessionRow[]>("telemetry_sessions") ?? [];
}

/**
 * Becomes: invoke("telemetry_sessions_page", { offset, limit, filter })
 *
 * A page at a time, so opening Telemetry costs one page rather than 300 rows.
 */
export function useSessionRowsPage(
 offset: number,
 limit = PAGE_SIZE,
): SessionRow[] {
 return (
  dataProvider().read?.<SessionRow[]>("telemetry_sessions_page", {
   offset,
   limit,
  }) ?? []
 );
}

/** Becomes: invoke("telemetry_sessions_count", { filter }) */
export function useSessionRowCount(): number {
 return dataProvider().read?.<number>("telemetry_sessions_count") ?? 0;
}

/** Becomes: invoke("telemetry_verb_counts") */
export function useVerbCounts(): VerbCount[] {
 return dataProvider().read?.<VerbCount[]>("telemetry_verb_counts") ?? [];
}

/** Becomes: invoke("telemetry_facets", { flat: true }) */
export function useFacets(): Facet2[] {
 return dataProvider().read?.<Facet2[]>("telemetry_facets_flat") ?? [];
}

/** Becomes: invoke("telemetry_spend") */
export function useSpend(): SpendRow[] {
 return dataProvider().read?.<SpendRow[]>("telemetry_spend") ?? [];
}
