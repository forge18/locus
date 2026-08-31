import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { TelemetryView } from "../../src/screens/review/TelemetryView";
import { useSessionRowCount } from "../../src/data/telemetry";

const mount = () => render(() => <TelemetryView />);
const rowsOf = (view: ReturnType<typeof mount>) => [
  ...view.getByTestId("tm-sessions-table-rows").querySelectorAll("tbody tr"),
];

describe("telemetry/filtering", () => {
  it("seeds aria-pressed from the fixture's active facet, then tracks clicks", async () => {
    const view = mount();
    const chip = view.getByTestId("facet-harness-claude");
    expect(chip.getAttribute("aria-pressed")).toBe("false");
    await fireEvent.click(chip);
    expect(chip.getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(chip);
    expect(chip.getAttribute("aria-pressed")).toBe("false");
  });

  it("filters the sessions to the harness facet the user picks", async () => {
    const view = mount();
    await fireEvent.click(view.getByTestId("facet-harness-codex"));
    const rows = rowsOf(view);
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) expect(row.textContent).toContain("codex");
    expect(
      view.getByTestId("tm-sessions-table-rows").getAttribute("data-total"),
    ).toBe(String(rows.length));
  });

  it("filters by an agent facet down to the one keeper session", async () => {
    const view = mount();
    await fireEvent.click(view.getByTestId("facet-agent_role-keeper-1"));
    const rows = rowsOf(view);
    expect(rows.length).toBe(1);
    expect(rows[0].textContent).toContain("keeper@1");
  });

  it("un-picking the facet brings the full page back", async () => {
    const view = mount();
    const chip = view.getByTestId("facet-harness-codex");
    await fireEvent.click(chip);
    await fireEvent.click(chip);
    expect(chip.getAttribute("aria-pressed")).toBe("false");
    expect(
      view.getByTestId("tm-sessions-table-rows").getAttribute("data-total"),
    ).toBe(String(useSessionRowCount()));
  });

  it("searches the loaded sessions as you type", async () => {
    const view = mount();
    await fireEvent.input(view.getByTestId("tm-query"), {
      target: { value: "aider" },
    });
    const rows = rowsOf(view);
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) expect(row.textContent).toContain("aider");
    expect(
      view.getByTestId("tm-sessions-table-rows").getAttribute("data-total"),
    ).toBe(String(rows.length));
  });

  it("shows the empty state when nothing matches", async () => {
    const view = mount();
    await fireEvent.input(view.getByTestId("tm-query"), {
      target: { value: "zzz-no-match" },
    });
    expect(view.getByTestId("tm-sessions-table-empty")).toBeTruthy();
    expect(view.queryByTestId("tm-sessions-table-rows")).toBeNull();
  });

  it("combines a facet and the search with AND", async () => {
    const view = mount();
    await fireEvent.click(view.getByTestId("facet-harness-claude"));
    await fireEvent.input(view.getByTestId("tm-query"), {
      target: { value: "aider" },
    });
    // A claude-only table cannot also contain an aider harness.
    expect(view.getByTestId("tm-sessions-table-empty")).toBeTruthy();
  });

  it("resets facets and search back to the full page", async () => {
    const view = mount();
    await fireEvent.click(view.getByTestId("facet-harness-codex"));
    await fireEvent.input(view.getByTestId("tm-query"), {
      target: { value: "codex" },
    });
    expect(
      view.getByTestId("facet-harness-codex").getAttribute("aria-pressed"),
    ).toBe("true");
    await fireEvent.click(view.getByTestId("tm-reset"));
    expect(
      view.getByTestId("facet-harness-codex").getAttribute("aria-pressed"),
    ).toBe("false");
    // The fixture-seeded active facet is cleared with the rest.
    expect(
      view.getByTestId("facet-verify-failed").getAttribute("aria-pressed"),
    ).toBe("false");
    expect((view.getByTestId("tm-query") as HTMLInputElement).value).toBe("");
    expect(
      view.getByTestId("tm-sessions-table-rows").getAttribute("data-total"),
    ).toBe(String(useSessionRowCount()));
  });

  it("leaves an event-only facet a toggle, without faking a row constraint", async () => {
    const view = mount();
    await fireEvent.click(view.getByTestId("facet-arbiter_class-bug"));
    expect(
      view.getByTestId("facet-arbiter_class-bug").getAttribute("aria-pressed"),
    ).toBe("true");
    // Arbiter classes classify events, not sessions, so the facet records the
    // choice without pretending to have narrowed the table.
    expect(
      view.getByTestId("tm-sessions-table-rows").getAttribute("data-total"),
    ).toBe(String(useSessionRowCount()));
  });
});
