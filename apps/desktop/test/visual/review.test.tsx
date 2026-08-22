import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { Shell } from "../../src/shell/Shell";
import { ArtifactsView } from "../../src/screens/review/ArtifactsView";
import { RunsView } from "../../src/screens/review/RunsView";
import { TelemetryView } from "../../src/screens/review/TelemetryView";
import { createNavStore } from "../../src/nav";
import type { View } from "../../src/nav";
import type { JSX } from "solid-js";
import { SRC, read, rules } from "../css";

/**
 * Structural conformance against screenshot 18 — not a pixel diff.
 * jsdom has no layout engine; what is asserted is what the screenshots encode
 * that survives without one.
 */
const SHOTS = resolve(SRC, "../../../docs/design_handoff_locus_v2/screenshots");
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;

const mount = (view: View, screen: () => JSX.Element) => {
  const nav = createNavStore({ view });
  return render(() => <Shell nav={nav}>{screen()}</Shell>);
};

describe("visual: review", () => {
  it("has the reference screenshot to conform to", () => {
    expect(existsSync(resolve(SHOTS, "18-review-telemetry.png"))).toBe(true);
  });

  it("lights Review with its three tabs, in order", () => {
    const { getByTestId } = mount("telemetry", () => <TelemetryView />);
    expect(getByTestId("rail-review").getAttribute("aria-current")).toBe(
      "true",
    );
    expect(
      [...getByTestId("tabbar-tabs").querySelectorAll(".tab")].map(
        (t) => t.textContent,
      ),
    ).toEqual(["Telemetry", "Runs", "Artifacts"]);
  });

  it("telemetry: search, chips, five cards, the three-column band, the sessions table", () => {
    const { getByTestId } = mount("telemetry", () => <TelemetryView />);
    expect(
      getByTestId("tm-metrics").querySelectorAll(".metric-card").length,
    ).toBe(5);
    expect(
      getByTestId("tm-sparkline").querySelectorAll(".sparkline-bar").length,
    ).toBe(16);
    expect(rule(".tm-band").body).toContain("repeat(3, minmax(0, 1fr))");
    expect(getByTestId("tm-sessions").querySelectorAll("th").length).toBe(11);
  });

  it("telemetry: the band holds Filters, Actions and Tools", () => {
    const { getByTestId } = mount("telemetry", () => <TelemetryView />);
    expect(
      [...getByTestId("tm-band").children].map((c) =>
        c.getAttribute("data-testid"),
      ),
    ).toEqual(["tm-filters", "tm-actions", "tm-tools"]);
  });

  it("runs: search, range control, count, three stats, the table", () => {
    const { getByTestId, container } = mount("runs", () => <RunsView />);
    expect(getByTestId("runs-search-note")).toBeTruthy();
    expect(container.querySelector(".seg")).not.toBe(null);
    expect(getByTestId("runs-stats").querySelectorAll(".run-stat").length).toBe(
      3,
    );
    expect(getByTestId("runs-table").querySelectorAll("th").length).toBe(10);
  });

  it("artifacts: three panes at 222 / flex / 306", () => {
    const { getByTestId } = mount("artifact", () => <ArtifactsView />);
    expect(
      (getByTestId("artifact-list") as HTMLElement).style.getPropertyValue(
        "--pane-w",
      ),
    ).toBe("222px");
    expect(
      (getByTestId("comment-rail") as HTMLElement).style.getPropertyValue(
        "--pane-w",
      ),
    ).toBe("306px");
    expect(getByTestId("artifact-view")).toBeTruthy();
  });

  it("carries the copy all three screenshots show, verbatim", () => {
    const telemetry = mount("telemetry", () => <TelemetryView />);
    expect(telemetry.getByTestId("tm-search-note").textContent).toBe(
      "every event, every session · BM25 over the normalized log",
    );
    expect(telemetry.getByTestId("permission-alarm").textContent).toContain(
      "is a misconfiguration alarm, not a metric",
    );
    telemetry.unmount();

    const artifacts = mount("artifact", () => <ArtifactsView />);
    expect(artifacts.getByTestId("artifact-group-reference").textContent).toBe(
      "Reference · never in the inbox",
    );
    expect(artifacts.getByTestId("artifact-one-viewer-note").textContent).toBe(
      "one viewer per kind · three entry points",
    );
  });

  it("paints every surface from a token", () => {
    expect(read("screens/screens.css")).not.toMatch(/#[0-9a-fA-F]{6}\b/);
  });
});
