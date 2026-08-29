import { For, Show, createMemo, createSignal } from "solid-js";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Input } from "../../ui/Input";
import { useTasks } from "../../data/board";
import { Segmented } from "../../ui/Segmented";
import {
        ANALYTICS_MEASURES,
        ANALYTICS_RANGES,
        useAnalyticsStats,
        useAtAGlanceMetrics,
        useBreakdown,
        useBreakdownDimensions,
        useExtensionKinds,
        useExtensionUsage,
        useRetrievalTiers,
        useTaskOutcomes,
        useTelemetryActions,
        useTelemetryFacets,
        useTelemetrySessions,
        useWorkflowTimings,
} from "../../data/analytics";
import type {
        AnalyticsMeasure,
        AnalyticsRange,
        AnalyticsScope,
        BreakdownDimension,
        ExtensionKind,
} from "../../data/analytics";

import "./analytics.css";

export interface AnalyticsViewProps {
        scope?: AnalyticsScope;
        initialTab?: "overview" | "telemetry";
}

export function AnalyticsView(props: AnalyticsViewProps) {
        const scope = () => props.scope ?? "all";
        const [tab, setTab] = createSignal(props.initialTab ?? "overview");
        const [range, setRange] = createSignal<AnalyticsRange>("30d");
        const [measure, setMeasure] = createSignal<AnalyticsMeasure>("spend");
        const [dimension, setDimension] =
                createSignal<BreakdownDimension>("Model");
        const [extensionKind, setExtensionKind] =
                createSignal<ExtensionKind>("all");
        const [search, setSearch] = createSignal("");
        const [selectedFacets, setSelectedFacets] = createSignal<string[]>([]);

        const query = createMemo(() => ({ scope: scope(), range: range() }));
        const stats = () => useAnalyticsStats(query());
        const atAGlance = () => useAtAGlanceMetrics(query());
        const ciStatus = () => {
                const tasks = useTasks();
                return {
                        passed: tasks.filter(
                                (task) => task.ciStatus === "passed",
                        ).length,
                        failed: tasks.filter(
                                (task) => task.ciStatus === "failed",
                        ).length,
                };
        };
        const breakdown = () => useBreakdown(query(), dimension(), measure());
        const trendBars = createMemo(() => {
                const values = breakdown().map((row) => {
                        const value =
                                measure() === "spend"
                                        ? row.spend
                                        : measure() === "tokens"
                                          ? row.tokens
                                          : measure() === "cache"
                                            ? row.cache
                                            : row.runs;
                        const parsed = Number.parseFloat(
                                String(value).replace(/[^0-9.]/g, ""),
                        );
                        return Number.isFinite(parsed) ? parsed : 0;
                });
                const source = values.length > 0 ? values : [0];
                const maximum = Math.max(...source, 1);
                const buckets =
                        ANALYTICS_RANGES.find(
                                (candidate) => candidate.value === range(),
                        )?.buckets ?? 30;
                const ceiling = Math.min(100, 50 + buckets * 1.5);
                return Array.from({ length: 10 }, (_, index) =>
                        Math.max(
                                12,
                                Math.round(
                                        (source[index % source.length] / maximum) *
                                                ceiling,
                                ),
                        ),
                );
        });
        const outcomes = () => useTaskOutcomes(query());
        const extensions = () => useExtensionUsage(query(), extensionKind());
        const facets = () => useTelemetryFacets(query(), search());
        const actions = () => useTelemetryActions(query(), search());
        const sessions = () => useTelemetrySessions(query(), search());
        const toggleFacet = (value: string) =>
                setSelectedFacets((current) =>
                        current.includes(value)
                                ? current.filter((item) => item !== value)
                                : [...current, value],
                );

        return (
                <div
                        class="analytics-view"
                        data-testid="analytics"
                        data-scope={scope()}
                >
                        <FixtureNotice
                                surface="Analytics"
                                command='invoke("analytics_aggregates")'
                        />
                        <header class="analytics-header">
                                <div>
                                        <h1>Analytics</h1>
                                        <p>
                                                {scope() === "all"
                                                        ? "All projects — the one surface that ignores the project selector."
                                                        : `#${scope()} — every projection scoped to this project.`}
                                        </p>
                                </div>
                                <Segmented
                                        options={[
                                                {
                                                        value: "overview",
                                                        label: "Overview",
                                                },
                                                {
                                                        value: "telemetry",
                                                        label: "Telemetry",
                                                },
                                        ]}
                                        value={tab()}
                                        onChange={(value) =>
                                                setTab(
                                                        value as
                                                                | "overview"
                                                                | "telemetry",
                                                )
                                        }
                                        label="Analytics view"
                                />
                        </header>
                        <div
                                class="analytics-range"
                                data-testid="analytics-range"
                        >
                                <Segmented
                                        options={ANALYTICS_RANGES.map(
                                                (item) => ({
                                                        value: item.value,
                                                        label: item.label,
                                                }),
                                        )}
                                        value={range()}
                                        onChange={(value) =>
                                                setRange(
                                                        value as AnalyticsRange,
                                                )
                                        }
                                        label="Analytics range"
                                />
                                <span>
                                        {
                                                ANALYTICS_RANGES.find(
                                                        (item) =>
                                                                item.value ===
                                                                range(),
                                                )?.buckets
                                        }{" "}
                                        buckets · all panels share this range
                                </span>
                        </div>

                        <Show
                                when={tab() === "overview"}
                                fallback={
                                        <TelemetryPanel
                                                query={query()}
                                                search={search()}
                                                setSearch={setSearch}
                                                selectedFacets={selectedFacets()}
                                                toggleFacet={toggleFacet}
                                                reset={() => {
                                                        setSelectedFacets([]);
                                                        setSearch("");
                                                }}
                                                facets={facets()}
                                                actions={actions()}
                                                sessions={sessions()}
                                        />
                                }
                        >
                                <main class="analytics-overview">
                                        <section
                                                class="analytics-stat-grid"
                                                data-testid="analytics-stat-cards"
                                        >
                                                <For each={stats()}>
                                                        {(stat) => (
                                                                <button
                                                                        type="button"
                                                                        class="analytics-stat"
                                                                        aria-pressed={
                                                                                measure() ===
                                                                                stat.id
                                                                        }
                                                                        onClick={() =>
                                                                                setMeasure(
                                                                                        stat.id,
                                                                                )
                                                                        }
                                                                        data-testid={`analytics-stat-${stat.id}`}
                                                                >
                                                                        <span>
                                                                                {
                                                                                        stat.label
                                                                                }
                                                                        </span>
                                                                        <strong>
                                                                                {
                                                                                        stat.value
                                                                                }
                                                                        </strong>
                                                                        <small>
                                                                                {
                                                                                        stat.note
                                                                                }
                                                                        </small>
                                                                </button>
                                                        )}
                                                </For>
                                        </section>
                                        <section
                                                class="analytics-metric-rail"
                                                data-testid="status-metrics"
                                        >
                                                <For each={atAGlance()}>
                                                        {(metric) => (
                                                                <div
                                                                        data-metric={
                                                                                metric.id
                                                                        }
                                                                >
                                                                        <span>
                                                                                {
                                                                                        metric.label
                                                                                }
                                                                        </span>
                                                                        <strong>
                                                                                {
                                                                                        metric.value
                                                                                }
                                                                        </strong>
                                                                        <small>
                                                                                {
                                                                                        metric.note
                                                                                }
                                                                        </small>
                                                                </div>
                                                        )}
                                                </For>
                                                <div
                                                        data-testid="status-ci-status"
                                                        data-metric="ci-status"
                                                >
                                                        <span>CI checks</span>
                                                        <strong>
                                                                {
                                                                        ciStatus()
                                                                                .passed
                                                                }{" "}
                                                                passed ·{" "}
                                                                {
                                                                        ciStatus()
                                                                                .failed
                                                                }{" "}
                                                                failing
                                                        </strong>
                                                        <small>
                                                                normalized forge
                                                                checks
                                                        </small>
                                                </div>
                                        </section>
                                        <section
                                                class="analytics-card analytics-trend"
                                                data-testid="analytics-trend"
                                        >
                                                <header>
                                                        <h2>Trend</h2>
                                                        <span>
                                                                Selected
                                                                measure:{" "}
                                                                {measure()}
                                                        </span>
                                                </header>
                                                <div class="analytics-measure-tabs">
                                                        <For
                                                                each={ANALYTICS_MEASURES.filter(
                                                                        (
                                                                                item,
                                                                        ) =>
                                                                                item.value !==
                                                                                "runs",
                                                                )}
                                                        >
                                                                {(item) => (
                                                                        <button
                                                                                type="button"
                                                                                aria-pressed={
                                                                                        measure() ===
                                                                                        item.value
                                                                                }
                                                                                onClick={() =>
                                                                                        setMeasure(
                                                                                                item.value as AnalyticsMeasure,
                                                                                        )
                                                                                }
                                                                        >
                                                                                {
                                                                                        item.label
                                                                                }
                                                                        </button>
                                                                )}
                                                        </For>
                                                </div>
                                                <div class="analytics-bars">
                                                        <For each={trendBars()} >
                                                                {(height) => (
                                                                        <i
                                                                                style={{
                                                                                        height: `${height}%`,
                                                                                }}
                                                                        />
                                                                )}
                                                        </For>
                                                </div>
                                        </section>
                                        <section
                                                class="analytics-card"
                                                data-testid="analytics-breakdown"
                                        >
                                                <header>
                                                        <h2>Breakdown</h2>
                                                        <span>
                                                                Same rows every
                                                                time — the bar
                                                                tracks the
                                                                measure you
                                                                picked.
                                                        </span>
                                                </header>
                                                <div class="analytics-dimension-tabs">
                                                        <For
                                                                each={useBreakdownDimensions()}
                                                        >
                                                                {(item) => (
                                                                        <button
                                                                                type="button"
                                                                                aria-pressed={
                                                                                        dimension() ===
                                                                                        item
                                                                                }
                                                                                onClick={() =>
                                                                                        setDimension(
                                                                                                item,
                                                                                        )
                                                                                }
                                                                        >
                                                                                {
                                                                                        item
                                                                                }
                                                                        </button>
                                                                )}
                                                        </For>
                                                </div>
                                                <table>
                                                        <thead>
                                                                <tr>
                                                                        <th>
                                                                                {dimension()}
                                                                        </th>
                                                                        <th>
                                                                                Tokens
                                                                        </th>
                                                                        <th>
                                                                                Cache
                                                                        </th>
                                                                        <th>
                                                                                Spend
                                                                        </th>
                                                                        <th>
                                                                                Runs
                                                                        </th>
                                                                        <th>
                                                                                Per
                                                                                run
                                                                        </th>
                                                                </tr>
                                                        </thead>
                                                        <tbody>
                                                                <For
                                                                        each={breakdown()}
                                                                >
                                                                        {(
                                                                                row,
                                                                        ) => (
                                                                                <tr>
                                                                                        <td>
                                                                                                {
                                                                                                        row.dimension
                                                                                                }
                                                                                        </td>
                                                                                        <td>
                                                                                                {
                                                                                                        row.tokens
                                                                                                }
                                                                                        </td>
                                                                                        <td>
                                                                                                {
                                                                                                        row.cache
                                                                                                }
                                                                                        </td>
                                                                                        <td>
                                                                                                {
                                                                                                        row.spend
                                                                                                }
                                                                                        </td>
                                                                                        <td>
                                                                                                {
                                                                                                        row.runs
                                                                                                }
                                                                                        </td>
                                                                                        <td>
                                                                                                {
                                                                                                        row.perRun
                                                                                                }
                                                                                        </td>
                                                                                </tr>
                                                                        )}
                                                                </For>
                                                        </tbody>
                                                </table>
                                        </section>
                                        <div class="analytics-two-col">
                                                <section
                                                        class="analytics-card"
                                                        data-testid="analytics-tasks"
                                                >
                                                        <header>
                                                                <h2>Tasks</h2>
                                                                <span>
                                                                        outcomes
                                                                        from
                                                                        board
                                                                        state
                                                                        and
                                                                        verify
                                                                        evidence
                                                                </span>
                                                        </header>
                                                        <div class="analytics-outcomes">
                                                                <For
                                                                        each={outcomes()}
                                                                >
                                                                        {(
                                                                                outcome,
                                                                        ) => (
                                                                                <div>
                                                                                        <span>
                                                                                                {
                                                                                                        outcome.label
                                                                                                }
                                                                                        </span>
                                                                                        <strong>
                                                                                                {
                                                                                                        outcome.count
                                                                                                }
                                                                                        </strong>
                                                                                </div>
                                                                        )}
                                                                </For>
                                                        </div>
                                                        <h3>Cost by role</h3>
                                                        <table>
                                                                <thead>
                                                                        <tr>
                                                                                <th>
                                                                                        Role
                                                                                </th>
                                                                                <th>
                                                                                        Landed
                                                                                </th>
                                                                                <th>
                                                                                        Cost
                                                                                </th>
                                                                                <th>
                                                                                        Runs
                                                                                </th>
                                                                                <th>
                                                                                        First
                                                                                        try
                                                                                </th>
                                                                        </tr>
                                                                </thead>
                                                                <tbody>
                                                                        <tr>
                                                                                <td>
                                                                                        builder
                                                                                </td>
                                                                                <td>
                                                                                        44
                                                                                </td>
                                                                                <td>
                                                                                        $82
                                                                                </td>
                                                                                <td>
                                                                                        188
                                                                                </td>
                                                                                <td>
                                                                                        76%
                                                                                </td>
                                                                        </tr>
                                                                        <tr>
                                                                                <td>
                                                                                        reviewer
                                                                                </td>
                                                                                <td>
                                                                                        22
                                                                                </td>
                                                                                <td>
                                                                                        $31
                                                                                </td>
                                                                                <td>
                                                                                        96
                                                                                </td>
                                                                                <td>
                                                                                        91%
                                                                                </td>
                                                                        </tr>
                                                                </tbody>
                                                        </table>
                                                        <p class="analytics-note">
                                                                Most expensive
                                                                to land:
                                                                migration audit
                                                                · 5 iterations ·
                                                                $14.20
                                                        </p>
                                                </section>
                                                <section
                                                        class="analytics-card"
                                                        data-testid="analytics-workflow-times"
                                                >
                                                        <header>
                                                                <h2>
                                                                        Run
                                                                        times by
                                                                        workflow
                                                                </h2>
                                                                <span>
                                                                        Median
                                                                        and p90
                                                                        wall-clock
                                                                        per run.
                                                                </span>
                                                        </header>
                                                        <table>
                                                                <thead>
                                                                        <tr>
                                                                                <th>
                                                                                        Workflow
                                                                                </th>
                                                                                <th>
                                                                                        Runs
                                                                                </th>
                                                                                <th>
                                                                                        Median
                                                                                </th>
                                                                                <th>
                                                                                        p90
                                                                                </th>
                                                                                <th>
                                                                                        Iter
                                                                                </th>
                                                                                <th>
                                                                                        Verified
                                                                                </th>
                                                                        </tr>
                                                                </thead>
                                                                <tbody>
                                                                        <For
                                                                                each={useWorkflowTimings(
                                                                                        query(),
                                                                                )}
                                                                        >
                                                                                {(
                                                                                        row,
                                                                                ) => (
                                                                                        <tr>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.workflow
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.runs
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.median
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.p90
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.iterations
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.verified
                                                                                                        }
                                                                                                </td>
                                                                                        </tr>
                                                                                )}
                                                                        </For>
                                                                </tbody>
                                                        </table>
                                                </section>
                                        </div>
                                        <div class="analytics-two-col">
                                                <section
                                                        class="analytics-card"
                                                        data-testid="analytics-memory"
                                                >
                                                        <header>
                                                                <h2>
                                                                        Memory
                                                                        retrievals
                                                                </h2>
                                                                <span>
                                                                        same
                                                                        range
                                                                        and
                                                                        scope
                                                                </span>
                                                        </header>
                                                        <table>
                                                                <thead>
                                                                        <tr>
                                                                                <th>
                                                                                        Tier
                                                                                </th>
                                                                                <th>
                                                                                        Hits
                                                                                </th>
                                                                                <th>
                                                                                        Useful
                                                                                </th>
                                                                                <th>
                                                                                        Average
                                                                                        tokens
                                                                                </th>
                                                                        </tr>
                                                                </thead>
                                                                <tbody>
                                                                        <For
                                                                                each={useRetrievalTiers(
                                                                                        query(),
                                                                                )}
                                                                        >
                                                                                {(
                                                                                        row,
                                                                                ) => (
                                                                                        <tr>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.tier
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.hits
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.useful
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.averageTokens
                                                                                                        }
                                                                                                </td>
                                                                                        </tr>
                                                                                )}
                                                                        </For>
                                                                </tbody>
                                                        </table>
                                                        <p class="analytics-note">
                                                                Recalls/run 3.4
                                                                · changed the
                                                                answer unknown ·
                                                                facts written 81
                                                                · promoted to
                                                                long-term 22
                                                        </p>
                                                        <strong>
                                                                Most read:
                                                                Guardrail
                                                                defaults · 143
                                                                reads
                                                        </strong>
                                                </section>
                                                <section
                                                        class="analytics-card"
                                                        data-testid="analytics-extensions"
                                                >
                                                        <header>
                                                                <h2>
                                                                        Extension
                                                                        usage
                                                                </h2>
                                                                <span>
                                                                        materialized
                                                                        or
                                                                        invoked,
                                                                        not
                                                                        definitions
                                                                </span>
                                                        </header>
                                                        <div class="analytics-dimension-tabs">
                                                                <For
                                                                        each={useExtensionKinds()}
                                                                >
                                                                        {(
                                                                                kind,
                                                                        ) => (
                                                                                <button
                                                                                        type="button"
                                                                                        aria-pressed={
                                                                                                extensionKind() ===
                                                                                                kind
                                                                                        }
                                                                                        onClick={() =>
                                                                                                setExtensionKind(
                                                                                                        kind,
                                                                                                )
                                                                                        }
                                                                                >
                                                                                        {
                                                                                                kind
                                                                                        }
                                                                                </button>
                                                                        )}
                                                                </For>
                                                        </div>
                                                        <table>
                                                                <thead>
                                                                        <tr>
                                                                                <th>
                                                                                        Extension
                                                                                </th>
                                                                                <th>
                                                                                        Hits
                                                                                </th>
                                                                                <th>
                                                                                        Note
                                                                                </th>
                                                                        </tr>
                                                                </thead>
                                                                <tbody>
                                                                        <For
                                                                                each={extensions()}
                                                                        >
                                                                                {(
                                                                                        row,
                                                                                ) => (
                                                                                        <tr>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.name
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.hits
                                                                                                        }
                                                                                                </td>
                                                                                                <td>
                                                                                                        {
                                                                                                                row.note
                                                                                                        }
                                                                                                </td>
                                                                                        </tr>
                                                                                )}
                                                                        </For>
                                                                </tbody>
                                                        </table>
                                                </section>
                                        </div>
                                </main>
                        </Show>
                </div>
        );
}

interface TelemetryPanelProps {
        query: { scope: AnalyticsScope; range: AnalyticsRange };
        search: string;
        setSearch: (value: string) => void;
        selectedFacets: string[];
        toggleFacet: (value: string) => void;
        reset: () => void;
        facets: ReturnType<typeof useTelemetryFacets>;
        actions: ReturnType<typeof useTelemetryActions>;
        sessions: ReturnType<typeof useTelemetrySessions>;
}
function TelemetryPanel(props: TelemetryPanelProps) {
        const maxAction = Math.max(
                ...props.actions.map((action) => action.count),
        );
        const metrics = useAtAGlanceMetrics(props.query);
        return (
                <main
                        class="telemetry-analytics"
                        data-testid="analytics-telemetry"
                >
                        <aside
                                class="analytics-facets"
                                data-testid="analytics-facets"
                        >
                                <h2>Filters</h2>
                                <For each={props.facets}>
                                        {(group) => (
                                                <div class="analytics-facet">
                                                        <span>
                                                                {group.label}
                                                        </span>
                                                        <For
                                                                each={
                                                                        group.values
                                                                }
                                                        >
                                                                {(value) => (
                                                                        <button
                                                                                type="button"
                                                                                aria-pressed={props.selectedFacets.includes(
                                                                                        value,
                                                                                )}
                                                                                onClick={() =>
                                                                                        props.toggleFacet(
                                                                                                value,
                                                                                        )
                                                                                }
                                                                        >
                                                                                {
                                                                                        value
                                                                                }
                                                                        </button>
                                                                )}
                                                        </For>
                                                </div>
                                        )}
                                </For>
                                <p>
                                        Every facet is a column on the
                                        normalized event log. Counts are the
                                        result set, not the corpus.
                                </p>
                        </aside>
                        <section class="analytics-telemetry-main">
                                <div class="analytics-search">
                                        <Input
                                                value={props.search}
                                                onInput={(event) =>
                                                        props.setSearch(
                                                                event
                                                                        .currentTarget
                                                                        .value,
                                                        )
                                                }
                                                placeholder="BM25 search over the normalized event log"
                                        />
                                        <Button
                                                variant="ghost"
                                                onClick={props.reset}
                                        >
                                                Reset filters
                                        </Button>
                                </div>
                                <div class="analytics-filter-chips">
                                        <For each={props.selectedFacets}>
                                                {(chip) => (
                                                        <button
                                                                type="button"
                                                                onClick={() =>
                                                                        props.toggleFacet(
                                                                                chip,
                                                                        )
                                                                }
                                                        >
                                                                {chip} ×
                                                        </button>
                                                )}
                                        </For>
                                </div>
                                <div class="analytics-stat-grid">
                                        <For
                                                each={[
                                                        ["Sessions", "641"],
                                                        ["Events", "154,385"],
                                                        [
                                                                "Tool errors",
                                                                "2,190",
                                                        ],
                                                        [
                                                                "Output tokens",
                                                                "77.46M",
                                                        ],
                                                ]}
                                        >
                                                {(metric) => (
                                                        <div class="analytics-stat">
                                                                <span>
                                                                        {
                                                                                metric[0]
                                                                        }
                                                                </span>
                                                                <strong>
                                                                        {
                                                                                metric[1]
                                                                        }
                                                                </strong>
                                                        </div>
                                                )}
                                        </For>
                                </div>
                                <section
                                        class="analytics-card analytics-metric-set"
                                        data-testid="telemetry-metrics"
                                >
                                        <header>
                                                <h2>Workflow metrics</h2>
                                                <span>
                                                        same rows, with facets
                                                        applied
                                                </span>
                                        </header>
                                        <For each={metrics}>
                                                {(metric) => (
                                                        <div
                                                                data-metric={
                                                                        metric.id
                                                                }
                                                        >
                                                                <span>
                                                                        {
                                                                                metric.label
                                                                        }
                                                                </span>
                                                                <strong>
                                                                        {
                                                                                metric.value
                                                                        }
                                                                </strong>
                                                                <small>
                                                                        {
                                                                                metric.note
                                                                        }
                                                                </small>
                                                        </div>
                                                )}
                                        </For>
                                </section>
                                <section class="analytics-card analytics-actions">
                                        <header>
                                                <h2>Actions</h2>
                                                <span>
                                                        canonical vocabulary ·
                                                        missing verbs stay
                                                        absent
                                                </span>
                                        </header>
                                        <For each={props.actions}>
                                                {(action) => (
                                                        <div
                                                                class={
                                                                        action.verb ===
                                                                        "permission_request"
                                                                                ? "analytics-action analytics-alarm"
                                                                                : "analytics-action"
                                                                }
                                                        >
                                                                <span>
                                                                        {
                                                                                action.verb
                                                                        }
                                                                </span>
                                                                <i>
                                                                        <b
                                                                                style={{
                                                                                        width: `${(action.count / maxAction) * 100}%`,
                                                                                }}
                                                                        />
                                                                </i>
                                                                <strong>
                                                                        {action.count.toLocaleString()}
                                                                </strong>
                                                        </div>
                                                )}
                                        </For>
                                        <p class="analytics-note">
                                                A nonzero permission_request
                                                count is a misconfiguration
                                                alarm, not a success metric.
                                        </p>
                                </section>
                                <section class="analytics-card">
                                        <header>
                                                <h2>Tools</h2>
                                                <span>
                                                        allowlisted payload with
                                                        anomaly notes
                                                </span>
                                        </header>
                                        <p class="analytics-note">
                                                Anomaly: researcher@1 ran
                                                web_fetch 4.1× its own baseline
                                                on 19 Aug.
                                        </p>
                                </section>
                                <section class="analytics-card">
                                        <h2>Sessions</h2>
                                        <table>
                                                <thead>
                                                        <tr>
                                                                <th>When</th>
                                                                <th>Harness</th>
                                                                <th>Project</th>
                                                                <th>repo</th>
                                                                <th>Agent</th>
                                                                <th>role</th>
                                                                <th>
                                                                        Model(s)
                                                                </th>
                                                                <th>Runs</th>
                                                                <th>Events</th>
                                                                <th>Errors</th>
                                                                <th>Tokens</th>
                                                                <th>Status</th>
                                                                <th>Id</th>
                                                        </tr>
                                                </thead>
                                                <tbody>
                                                        <For
                                                                each={
                                                                        props.sessions
                                                                }
                                                        >
                                                                {(row) => (
                                                                        <tr>
                                                                                <td>
                                                                                        {
                                                                                                row.when
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.harness
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.project
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.repo
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.agent
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.role
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.models
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.runs
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.events
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.errors
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.tokens
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.status
                                                                                        }
                                                                                </td>
                                                                                <td>
                                                                                        {
                                                                                                row.id
                                                                                        }
                                                                                </td>
                                                                        </tr>
                                                                )}
                                                        </For>
                                                </tbody>
                                        </table>
                                </section>
                        </section>
                </main>
        );
}

export default AnalyticsView;
