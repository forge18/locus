import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WorkshopFixtureView } from "../../src/screens/workshop/WorkshopFixtureView";

describe("Workshop autorouting", () => {
  it("renders six routing bands and fallback", () => {
    const { getAllByTestId, getByTestId } = render(() => (
      <WorkshopFixtureView fixture="harnesses" />
    ));
    expect(getAllByTestId(/autoroute-band-/)).toHaveLength(6);
    expect(getByTestId("autoroute-fallback").textContent).toContain(
      "falls upward",
    );
  });

  it("switches autorouting off to harness defaults", () => {
    const { getByTestId, queryAllByTestId } = render(() => (
      <WorkshopFixtureView fixture="harnesses" />
    ));
    const toggle = getByTestId("autorouting-toggle");
    expect(toggle.getAttribute("aria-pressed")).toBe("true");
    toggle.click();
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    expect(getByTestId("autoroute-disabled").textContent).toContain(
      "harness default",
    );
    expect(queryAllByTestId(/autoroute-band-/)).toHaveLength(0);
  });
});
