import { For, createSignal } from "solid-js";
import { Icon } from "../../ui/Icon";
import { Segmented } from "../../ui/Segmented";
import { VirtualTable } from "../../panes/VirtualTable";
import type { Column } from "../../ui/Table";
import {
  DEFAULT_RANGE,
  PAGE_SIZE,
  RANGES,
  SEARCH_NOTE,
  useRunCount,
  useRunStats,
  useRunsPage,
} from "../../data/runs";
import type { RunRow } from "../../data/runs";

const unknown = () => <span class="unknown">unknown</span>;

const orUnknown = (value: number | null) =>
  value === null ? unknown() : `${(value / 1000).toFixed(1)}k`;

const COLUMNS: Column<RunRow>[] = [
  {
    key: "at",
    header: "When ↓",
    type: "mono",
    cell: (r) => r.at.slice(0, 16).replace("T", " "),
  },
  { key: "harness", header: "Harness", cell: (r) => r.harness },
  {
    key: "project",
    header: "Project · repo",
    cell: (r) => `${r.project} · core`,
  },
  {
    key: "agent",
    header: "Agent · role",
    type: "mono",
    cell: (r) => `${r.agent} · ${r.role}`,
  },
  // The model that answered, not the tier that was asked for.
  {
    key: "model",
    header: "Model resolved",
    type: "mono",
    cell: (r) => r.model,
  },
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
  { key: "cache", header: "Cache", type: "numeric", cell: () => unknown() },
  { key: "spend", header: "Spend", type: "numeric", cell: () => unknown() },
  {
    key: "verify",
    header: "Verify",
    cell: (r) => (
      <span
        class={
          r.status === "passed"
            ? "verify-ok"
            : r.status === "failed"
              ? "verify-bad"
              : ""
        }
      >
        {r.status}
      </span>
    ),
  },
  { key: "id", header: "Id", type: "mono", cell: (r) => r.id },
];

/** Rows are 26px; the body is drawn at 420px. Both are what the window is sized from. */
const ROW_HEIGHT = 26;
const BODY_HEIGHT = 420;

export function RunsView() {
  const [range, setRange] = createSignal<string>(DEFAULT_RANGE);
  const total = useRunCount();
  const [loaded, setLoaded] = createSignal(useRunsPage(0));

  /** One page at a time, as the window approaches the end of what is loaded. */
  const loadMore = () => {
    if (loaded().length >= total) return;
    setLoaded([...loaded(), ...useRunsPage(loaded().length, PAGE_SIZE)]);
  };

  return (
    <div class="runs" data-testid="runs">
      <div class="runs-head" data-testid="runs-head">
        <div
          class="tm-search"
          style={{ flex: "1", "max-width": "420px" }}
          data-testid="runs-search"
        >
          <Icon
            name="magnifying-glass"
            size={12}
            style={{ color: "var(--text-muted)" }}
          />
          <span
            class="tm-search-note"
            style={{ "margin-left": "0" }}
            data-testid="runs-search-note"
          >
            {SEARCH_NOTE}
          </span>
        </div>

        <Segmented
          options={RANGES.map((r) => ({ value: r.value, label: r.label }))}
          value={range()}
          onChange={setRange}
          label="Range"
        />

        <span class="runs-count" data-testid="runs-count">
          {total.toLocaleString("en-US")} runs
        </span>

        <div class="runs-stats" data-testid="runs-stats">
          <For each={useRunStats()}>
            {(stat) => (
              <div
                class="run-stat"
                data-testid={`run-stat-${stat.label.replace(/\W+/g, "-")}`}
              >
                <span class="run-stat-value">{stat.value}</span>
                <span class="run-stat-label">{stat.label}</span>
              </div>
            )}
          </For>
        </div>
      </div>

      <section class="panel" data-testid="runs-panel">
        <span class="panel-title">Runs ({total})</span>
        <VirtualTable
          testId="runs-table"
          columns={COLUMNS}
          rows={loaded()}
          total={total}
          rowKey={(r) => r.id}
          rowHeight={ROW_HEIGHT}
          height={BODY_HEIGHT}
          onLoadMore={loadMore}
        />
      </section>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default RunsView;
