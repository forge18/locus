import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import { isTauri } from "@tauri-apps/api/core";
import { InlineError } from "../../ui/InlineError";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Icon } from "../../ui/Icon";
import { VirtualTable } from "../../panes/VirtualTable";
import type { Column } from "../../ui/Table";
import {
  ACTION_NOTE,
  fetchTelemetryMetrics,
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
} from "../../data/telemetry";
import type { SessionRow, TelemetryMetric } from "../../data/telemetry";
import { failed, type Envelope } from "../../data/envelope";

const orUnknown = (value: string | null) =>
  value === null ? <span class="unknown">unknown</span> : value;

const COLUMNS: Column<SessionRow>[] = [
  { key: "when", header: "When ↓", type: "mono", cell: (r) => r.when },
  { key: "harness", header: "Harness", cell: (r) => r.harness },
  {
    key: "project",
    header: "Project · repo",
    cell: (r) => `${r.project} · ${r.repo}`,
  },
  {
    key: "agent",
    header: "Agent · role",
    type: "mono",
    cell: (r) => `${r.agent} · ${r.role}`,
  },
  { key: "models", header: "Model(s)", type: "mono", cell: (r) => r.models },
  { key: "runs", header: "Runs", type: "numeric", cell: (r) => r.runs },
  {
    key: "events",
    header: "Events",
    type: "numeric",
    cell: (r) => r.events.toLocaleString("en-US"),
  },
  {
    key: "errors",
    header: "Errors",
    type: "numeric",
    cell: (r) => (
      <span class={r.errors > 0 ? "verify-bad" : ""}>{r.errors}</span>
    ),
  },
  {
    key: "tokens",
    header: "Tokens",
    type: "numeric",
    cell: (r) => orUnknown(r.tokens),
  },
  {
    key: "status",
    header: "Status",
    cell: (r) => (
      <span class={`status-${r.status.replace(/\s+/g, "-")}`}>
        {r.statusDetail ? `${r.status} ${r.statusDetail}` : r.status}
      </span>
    ),
  },
  { key: "id", header: "Id", type: "mono", cell: (r) => r.id },
];

/**
 * Facet counts come from the event corpus, and a session carries only some of
 * the same dimensions. A group mapped here filters the table; the rest (capture
 * source, model tier, verify, arbiter class, branch) still toggle and reset —
 * they just cannot constrain these rows.
 */
const SESSION_FACET_FIELD: Record<string, (row: SessionRow) => string> = {
  harness: (r) => r.harness,
  project: (r) => r.project,
  agent_role: (r) => r.agent,
};

/** What a search query matches: the text fields the table renders. */
const rowText = (row: SessionRow): string =>
  [
    row.when,
    row.harness,
    row.project,
    row.repo,
    row.agent,
    row.role,
    row.models,
    row.status,
    row.statusDetail ?? "",
    row.tokens ?? "",
    row.id,
  ]
    .join(" ")
    .toLowerCase();

/**
 * Review is now; Analytics is after. Every number here is already a column, so
 * this screen is a query rather than new instrumentation.
 */
/** Rows are 26px; the table body is drawn at 300px. */
const ROW_HEIGHT = 26;
const BODY_HEIGHT = 300;

function TelemetryLive(props: { projectId?: string }) {
  const [metrics, setMetrics] = createSignal<Envelope<TelemetryMetric[]>>({
    status: "loading",
  });
  const rows = createMemo(() => {
    const state = metrics();
    return state.status === "ready" ? state.data : [];
  });
  const errorMessage = createMemo(() => {
    const state = metrics();
    return state.status === "failed"
      ? `${state.error.command}: ${state.error.message}`
      : "";
  });

  let requestId = 0;
  createEffect(() => {
    const scope = props.projectId ?? "all";
    const request = ++requestId;
    setMetrics({ status: "loading" });
    void fetchTelemetryMetrics(scope, "30d")
      .then((result) => {
        if (request === requestId) setMetrics(result);
      })
      .catch((cause) => {
        if (request === requestId) setMetrics(failed("telemetry_metrics", cause));
      });
  });

  return (
    <div
      class="telemetry"
      data-testid="telemetry"
      data-desktop-route="review-telemetry"
      data-filter-evidence="available"
    >
      <main data-testid="telemetry-live-state" data-state={metrics().status}>
        <Switch>
          <Match when={metrics().status === "loading"}>
            <p>Loading telemetry…</p>
          </Match>
          <Match when={metrics().status === "failed"}>
            <InlineError
              cause={errorMessage()}
              next="Check the project connection and retry Telemetry."
            />
          </Match>
          <Match when={metrics().status === "empty"}>
            <p>No telemetry in this scope.</p>
          </Match>
          <Match when={metrics().status === "ready"}>
            <section class="tm-metrics" data-testid="tm-live-metrics">
              <For each={rows()}>
                {(metric) => (
                  <div
                    class={[
                      "metric-card",
                      metric.bad ? "tm-metric-bad" : "",
                    ]
                      .filter(Boolean)
                      .join(" ")}
                    data-bad={metric.bad ? "true" : undefined}
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
            </section>
          </Match>
        </Switch>
      </main>
    </div>
  );
}

export interface TelemetryViewProps {
  projectId?: string;
}

export function TelemetryView(props: TelemetryViewProps = {}) {
  if (isTauri()) return <TelemetryLive projectId={props.projectId} />;

  const sessionTotal = useSessionRowCount();
  const [loaded, setLoaded] = createSignal(useSessionRowsPage(0));

  /** One page at a time, as the window approaches the end of what is loaded. */
  const loadMore = () => {
    if (loaded().length >= sessionTotal) return;
    setLoaded([...loaded(), ...useSessionRowsPage(loaded().length, PAGE_SIZE)]);
  };

  const actions = useActionRows();
  const tools = useToolRows();
  const maxAction = Math.max(...actions.map((a) => a.count));
  const maxTool = Math.max(...tools.map((t) => t.count));

  /** The box starts empty; the fixture query is its placeholder. */
  const [query, setQuery] = createSignal("");

  /** Seeded with the fixture's active facet; every click takes over from there. */
  const [activeFacets, setActiveFacets] = createSignal(
    new Set(
      useFacetGroups().flatMap((group) =>
        group.facets
          .filter((facet) => facet.active)
          .map((facet) => `${group.key}:${facet.value}`),
      ),
    ),
  );

  /** The facet values picked under one group. */
  const picked = (groupKey: string): string[] =>
    [...activeFacets()]
      .filter((key) => key.startsWith(`${groupKey}:`))
      .map((key) => key.slice(groupKey.length + 1));

  const toggleFacet = (groupKey: string, value: string) => {
    const key = `${groupKey}:${value}`;
    const next = new Set(activeFacets());
    if (next.has(key)) next.delete(key);
    else next.add(key);
    setActiveFacets(next);
  };

  const resetFilters = () => {
    setQuery("");
    setActiveFacets(new Set<string>());
  };

  /** A filter is live only while it can constrain these rows. */
  const filtering = () =>
    query().trim().length > 0 ||
    Object.keys(SESSION_FACET_FIELD).some((groupKey) => picked(groupKey).length > 0);

  /** The loaded page, narrowed by the live facet and search filters. */
  const rows = () => {
    const q = query().trim().toLowerCase();
    const facetFilters = Object.entries(SESSION_FACET_FIELD).map(([groupKey, field]) => ({
      field,
      chosen: picked(groupKey),
    }));
    if (q.length === 0 && facetFilters.every((f) => f.chosen.length === 0)) return loaded();
    return loaded().filter(
      (row) =>
        (q.length === 0 || rowText(row).includes(q)) &&
        facetFilters.every(
          ({ field, chosen }) => chosen.length === 0 || chosen.includes(field(row)),
        ),
    );
  };

  return (
    <div
      class="telemetry"
      data-testid="telemetry"
      data-desktop-route="review-telemetry"
      data-filter-evidence="available"
    >
      <FixtureNotice
        surface="Telemetry"
        command='invoke("telemetry_aggregates")'
      />
      <div class="tm-search" data-testid="tm-search">
        <Icon
          name="magnifying-glass"
          size={12}
          style={{ color: "var(--text-muted)" }}
        />
        <input
          class="tm-query"
          type="search"
          value={query()}
          placeholder={SEARCH_QUERY}
          aria-label="Search telemetry"
          data-testid="tm-query"
          style={{ background: "transparent", border: "0", padding: "0" }}
          onInput={(event) => setQuery(event.currentTarget.value)}
        />
        <span class="tm-search-note" data-testid="tm-search-note">
          {SEARCH_NOTE}
        </span>
      </div>

      <div class="tm-chips" data-testid="tm-chips">
        <For each={useFilterChips()}>
          {(chip) => (
            <span
              class="tag tag-outline"
              data-testid={`tm-chip-${chip.label.replace(/\W+/g, "-")}`}
              data-active={chip.active ? "true" : undefined}
            >
              {chip.label}
            </span>
          )}
        </For>
        <button
          type="button"
          class="tm-reset"
          data-testid="tm-reset"
          onClick={resetFilters}
        >
          {RESET_LABEL}
        </button>
      </div>

      <div class="tm-metrics" data-testid="tm-metrics">
        <For each={useTelemetryMetrics()}>
          {(metric) => (
            <div
              class={["metric-card", metric.bad ? "tm-metric-bad" : ""]
                .filter(Boolean)
                .join(" ")}
              data-testid={`tm-metric-${metric.label.toLowerCase().replace(/\s+/g, "-")}`}
              data-bad={metric.bad ? "true" : undefined}
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
                        class={[
                          "facet",
                          facet.invariant ? "facet-invariant" : "",
                        ]
                          .filter(Boolean)
                          .join(" ")}
                        data-testid={`facet-${group.key}-${facet.value.replace(/\W+/g, "-")}`}
                        aria-pressed={
                          activeFacets().has(`${group.key}:${facet.value}`)
                            ? "true"
                            : "false"
                        }
                        onClick={() => toggleFacet(group.key, facet.value)}
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
                          style={{
                            width: `${(action.count / maxAction) * 100}%`,
                          }}
                        />
                      </span>
                      <span class="bar-count">
                        {action.count.toLocaleString("en-US")}
                      </span>
                    </div>
                    <div class="alarm-callout" data-testid="permission-alarm">
                      <Icon
                        name="warning-octagon"
                        weight="fill"
                        size={12}
                        style={{
                          "flex-shrink": 0,
                          color: "var(--status-danger)",
                        }}
                      />
                      <span>{action.alarm}</span>
                    </div>
                  </>
                }
              >
                <div
                  class={["bar-row", action.bad ? "bar-row-bad" : ""]
                    .filter(Boolean)
                    .join(" ")}
                  data-testid={`action-${action.verb}`}
                >
                  <span class="bar-label">{action.verb}</span>
                  <span class="bar-track">
                    <span
                      class={["bar-fill", action.bad ? "bar-fill-bad" : ""]
                        .filter(Boolean)
                        .join(" ")}
                      style={{ width: `${(action.count / maxAction) * 100}%` }}
                    />
                  </span>
                  <span class="bar-count">
                    {action.count.toLocaleString("en-US")}
                  </span>
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
              <div
                class="bar-row"
                data-testid={`tool-${tool.tool.replace(/\W+/g, "-")}`}
              >
                <span class="bar-label bar-label-tool">{tool.tool}</span>
                <span class="bar-track">
                  <span
                    class="bar-fill"
                    style={{ width: `${(tool.count / maxTool) * 100}%` }}
                  />
                </span>
                <span class="bar-count">
                  {tool.count.toLocaleString("en-US")}
                </span>
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
          rows={rows()}
          total={filtering() ? rows().length : sessionTotal}
          rowKey={(r) => r.id}
          rowHeight={ROW_HEIGHT}
          height={BODY_HEIGHT}
          onLoadMore={filtering() ? undefined : loadMore}
        />
      </section>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default TelemetryView;
