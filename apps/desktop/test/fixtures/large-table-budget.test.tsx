import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Table, type Column } from "../../src/ui/Table";
import { VirtualTable } from "../../src/panes/VirtualTable";
import {
  PAGE_SIZE,
  fetchRunsPage,
  type DispatchRunRow,
} from "../../src/data/runs";
import type { DataProvider } from "../../src/data/provider";
import { configureDataProvider } from "../../src/data/provider";

const ROW_HEIGHT = 26;
const BODY_HEIGHT = 420;
const TOTAL = 612;

/** 612 store-shaped runs — the size that justifies the windowing budget. */
const RUNS: DispatchRunRow[] = Array.from({ length: TOTAL }, (_, index) => ({
  id: `run-${String(index).padStart(4, "0")}`,
  project: index % 2 === 0 ? "tapestry" : "loom-db",
  agent: "builder",
  branch: "agent/run",
  status: index % 5 === 0 ? "failed" : "completed",
  harness: "claude",
  role: "builder",
  model: "claude-opus-4",
  events: 12,
  errors: index % 5 === 0 ? 1 : 0,
  startedAt: "2026-08-30T12:00:00Z",
}));

const COLUMNS: Column<DispatchRunRow>[] = [
  { key: "id", header: "Run", type: "mono", cell: (r) => r.id },
  { key: "project", header: "Project", cell: (r) => r.project },
  { key: "agent", header: "Agent", cell: (r) => r.agent },
  { key: "branch", header: "Branch", type: "mono", cell: (r) => r.branch },
  { key: "status", header: "Status", cell: (r) => r.status },
  { key: "model", header: "Model", type: "mono", cell: (r) => r.model },
  { key: "events", header: "Events", type: "numeric", cell: (r) => String(r.events) },
  { key: "errors", header: "Errors", type: "numeric", cell: (r) => String(r.errors) },
  { key: "startedAt", header: "At", type: "mono", cell: (r) => r.startedAt ?? "—" },
];

/** Page one of the live read: what a first paint waits for. */
const firstPage = (): DispatchRunRow[] => RUNS.slice(0, PAGE_SIZE);

const virtual = () =>
  render(() => (
    <VirtualTable
      columns={COLUMNS}
      rows={firstPage()}
      total={RUNS.length}
      rowKey={(r) => r.id}
      rowHeight={ROW_HEIGHT}
      height={BODY_HEIGHT}
    />
  ));

describe("fixtures/large-table-budget", () => {
  it("the live read asks the host for exactly one page", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const provider: DataProvider = {
      kind: "demo",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: firstPage() as T[] };
      },
      async queryOne<T>(command: string) {
        calls.push({ command });
        return { status: "ready", data: TOTAL as T };
      },
    };
    configureDataProvider(provider);

    const envelope = await fetchRunsPage(0);
    expect(envelope.status).toBe("ready");
    expect(calls).toEqual([
      { command: "dispatch_runs_page", args: { offset: 0, limit: PAGE_SIZE } },
    ]);
    expect(
      envelope.status === "ready" ? envelope.data.length : 0,
    ).toBe(PAGE_SIZE);
  });

  it("opens on one page, not on all 612 rows", () => {
    const { getByTestId } = virtual();
    const rows = getByTestId("table-rows");
    expect(rows.getAttribute("data-total")).toBe(String(TOTAL));
    expect(rows.getAttribute("data-loaded")).toBe(String(PAGE_SIZE));
  });

  it("renders only the window, which is a fraction of the page", () => {
    const { getByTestId } = virtual();
    const rendered = getByTestId("table-rows").querySelectorAll("tbody tr").length;
    const window_ = Math.ceil(BODY_HEIGHT / ROW_HEIGHT);
    expect(rendered).toBeGreaterThan(0);
    expect(rendered).toBeLessThan(window_ * 2 + 20);
    expect(rendered).toBeLessThan(RUNS.length / 4);
  });

  it("costs a fraction of the nodes the full table would", () => {
    const windowed = virtual();
    const windowedNodes = windowed.getByTestId("table").querySelectorAll("*").length;
    windowed.unmount();

    const full = render(() => (
      <Table columns={COLUMNS} rows={RUNS} rowKey={(r) => r.id} />
    ));
    const fullNodes = full.getByTestId("table").querySelectorAll("*").length;

    // Both numbers on the record, and the ratio is why the window exists.
    expect(windowedNodes).toBeLessThan(fullNodes / 4);
  });

  it("keeps the scrollbar honest with spacers for the rows it did not render", () => {
    const { getByTestId } = virtual();
    const rows = getByTestId("table-rows");
    const first = Number(rows.getAttribute("data-first"));
    const last = Number(rows.getAttribute("data-last"));
    const top = getByTestId("virtual-spacer-top") as HTMLElement;
    const bottom = getByTestId("virtual-spacer-bottom") as HTMLElement;

    expect(top.style.height).toBe(`${first * ROW_HEIGHT}px`);
    expect(bottom.style.height).toBe(`${(TOTAL - last) * ROW_HEIGHT}px`);

    // Spacers plus rendered rows add up to the whole list, so the scrollbar is
    // the size it would be if every row were there.
    const total = first * ROW_HEIGHT + (last - first) * ROW_HEIGHT + (TOTAL - last) * ROW_HEIGHT;
    expect(total).toBe(TOTAL * ROW_HEIGHT);
  });

  it("says how much is loaded while pages are still coming", () => {
    const { getByTestId } = virtual();
    expect(getByTestId("table-loading").textContent).toContain(
      `${PAGE_SIZE} of ${TOTAL} loaded`,
    );
  });

  it("distinguishes an empty result from an initial loading page", () => {
    const empty = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ));
    expect(empty.getByTestId("table").getAttribute("data-state")).toBe("empty");
    expect(empty.getByTestId("table-empty").textContent).toContain("No rows to display.");
    empty.unmount();

    const loading = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        loading
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ));
    expect(loading.getByTestId("table").getAttribute("data-state")).toBe("loading");
    expect(loading.getByTestId("table-loading").textContent).toBe("Loading…");
    expect(loading.getByTestId("skeleton-rows")).toBeTruthy();
  });

  it("keeps a backend error distinct from an empty result", () => {
    const { getByTestId, queryByTestId } = render(() => (
      <VirtualTable
        columns={COLUMNS}
        rows={[]}
        total={0}
        error="Runs unavailable"
        rowKey={(r) => r.id}
        rowHeight={ROW_HEIGHT}
        height={BODY_HEIGHT}
      />
    ));
    expect(getByTestId("table").getAttribute("data-state")).toBe("error");
    expect(getByTestId("inline-error-cause").textContent).toBe("Runs unavailable");
    expect(queryByTestId("table-empty")).toBe(null);
  });
});
