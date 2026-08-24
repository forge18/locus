import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TelemetryView } from "../../src/screens/review/TelemetryView";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;
const mount = () => render(() => <TelemetryView />);

describe("telemetry/layout", () => {
  it("is a scrolling column", () => {
    const body = rule(".telemetry").body;
    expect(body).toContain("flex-direction: column");
    expect(body).toContain("overflow: auto");
  });

  it("stacks search, chips, metrics, the band, then the table", () => {
    const { getByTestId } = mount();
    expect(
      [...getByTestId("telemetry").children].map((c) =>
        c.getAttribute("data-testid"),
      ),
    ).toEqual([
      "tm-search",
      "tm-chips",
      "tm-metrics",
      "tm-band",
      "tm-sessions",
    ]);
  });

  it("carries no per-screen colours", () => {
    expect(read("screens/review/TelemetryView.tsx")).not.toMatch(
      /#[0-9a-fA-F]{6}\b/,
    );
  });

  it("is a query over what is already recorded, not new instrumentation", () => {
    // Everything on the screen comes from an accessor; nothing computes a metric.
    const source = read("screens/review/TelemetryView.tsx");
    expect(source).toMatch(/from ["']\.\.\/\.\.\/data\/telemetry["']/);
    expect(source).not.toMatch(/setInterval|Date\.now/);
  });
});
