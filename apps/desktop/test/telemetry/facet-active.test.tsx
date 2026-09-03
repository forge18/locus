import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TelemetryView } from "../../src/screens/review/TelemetryView";
import { useFacetGroups } from "../../src/data/telemetry";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel)!;
const mount = () => render(() => <TelemetryView />);

describe("telemetry/facet-active", () => {
  it("marks the active facet in the DOM", () => {
    const { getByTestId } = mount();
    expect(
      getByTestId("facet-verify-failed").getAttribute("aria-pressed"),
    ).toBe("true");
    expect(
      getByTestId("facet-verify-passed").getAttribute("aria-pressed"),
    ).toBe("false");
  });

  it("uses a contrasting accent fill and an inset ring", () => {
    const body = rule('.facet[aria-pressed="true"]').body;
    expect(body).toContain("background: var(--action-attention)");
    expect(body).toContain("box-shadow: var(--ring-sel-soft)");
    expect(body).toContain("color: var(--action-attention-ink)");
  });

  it("uses contrasting ink for its count too", () => {
    expect(rule('.facet[aria-pressed="true"] .facet-count').body).toContain(
      "color: var(--action-attention-ink)",
    );
  });

  it("has exactly one active facet in the fixture, matching the filter chip", () => {
    const active = useFacetGroups()
      .flatMap((g) => g.facets)
      .filter((f) => f.active);
    expect(active.length).toBe(1);
    expect(active[0].value).toBe("failed");
  });

  it("never uses an outer glow to say active — the ring token is inset", () => {
    expect(rule('.facet[aria-pressed="true"]').body).toContain(
      "box-shadow: var(--ring-sel-soft)",
    );
    expect(read("styles/tokens.css")).toContain(
      "--ring-sel-soft: inset 0 0 0 1px var(--action-attention-ring)",
    );
  });
});
