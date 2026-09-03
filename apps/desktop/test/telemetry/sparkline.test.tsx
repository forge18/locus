import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TelemetryView } from "../../src/screens/review/TelemetryView";
import { useSparkline } from "../../src/data/telemetry";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;
const mount = () => render(() => <TelemetryView />);

describe("telemetry/sparkline", () => {
  it("draws sixteen bars", () => {
    const { getByTestId } = mount();
    expect(
      getByTestId("tm-sparkline").querySelectorAll(".sparkline-bar").length,
    ).toBe(16);
    expect(useSparkline().length).toBe(16);
  });

  it("draws them with a muted data-ramp color", () => {
    const body = rule(".sparkline-bar").body;
    expect(body).toContain(
      "background: color-mix(in srgb, var(--data-2) 85%, var(--surface-raised))",
    );
    expect(body).not.toMatch(/opacity:\s*\.85/);
  });

  it("sizes each bar from the data", () => {
    const { getByTestId } = mount();
    const heights = [
      ...getByTestId("tm-sparkline").querySelectorAll(".sparkline-bar"),
    ].map((b) => (b as HTMLElement).style.height);
    expect(heights).toEqual(useSparkline().map((v) => `${v}%`));
  });

  it("grows from the bottom", () => {
    expect(rule(".sparkline").body).toContain("align-items: flex-end");
  });

  it("is deterministic — the same 16 bars on every render", () => {
    expect(useSparkline()).toEqual(useSparkline());
  });
});
