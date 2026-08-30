import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import {
  HEADER_NOTE,
  HarnessesView,
} from "../../src/screens/workshop/HarnessesView";
import { useHarnessSummary } from "../../src/data/harnesses";
import { read, rules } from "../css";

const mount = () => render(() => <HarnessesView />);

describe("harnesses/header", () => {
  it('reads "Registered harnesses N", counted from the registry', () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-title").textContent).toContain(
      "Registered harnesses",
    );
    expect(getByTestId("harnesses-count").textContent).toBe(
      String(useHarnessSummary().harnesses),
    );
  });

  it("says mechanism lives in the file and policy lives here", () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-note").textContent).toBe(HEADER_NOTE);
    expect(HEADER_NOTE).toContain(
      "Mechanism lives in the file; policy lives here",
    );
    expect(HEADER_NOTE).toContain("only the mechanism differs");
  });

  it("carries a two-part legend", () => {
    const { getByTestId } = mount();
    const legend = getByTestId("harnesses-legend");
    expect(legend.textContent).toContain("native");
    expect(legend.textContent).toContain("downgraded — each names its loss");
  });

  it("colours the legend swatches accent and the downgrade red", () => {
    const css = rules(read("screens/screens.css"));
    expect(
      css.find((r) => r.selector === ".hn-legend-native i")!.body,
    ).toContain("background: var(--action-attention)");
    expect(
      css.find((r) => r.selector === ".hn-legend-downgraded i")!.body,
    ).toContain(
      "background: color-mix(in srgb, var(--status-danger) 55%, transparent)",
    );
  });

  it("offers Register a harness as the primary", () => {
    const { getByTestId } = mount();
    expect(getByTestId("harnesses-register").textContent).toContain(
      "Register a harness",
    );
    expect(getByTestId("harnesses-register").className).toContain(
      "btn-primary",
    );
  });
});
