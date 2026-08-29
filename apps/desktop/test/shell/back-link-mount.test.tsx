import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";

describe("shell/back-link-mount", () => {
  it("shows no back link on a view the rail lands on", () => {
    const { queryByTestId } = render(() => (
      <Shell nav={createNavStore({ view: "inbox" })}>
        <div />
      </Shell>
    ));
    expect(queryByTestId("drilldown-back")).toBeNull();
  });

  it("shows the drill-down back link above the screen and works it", () => {
    const nav = createNavStore({ view: "canvas" });
    const { getByTestId, queryByTestId } = render(() => (
      <Shell nav={nav}>
        <div />
      </Shell>
    ));
    const link = getByTestId("drilldown-back");
    expect(link.textContent).toContain("Workflows");
    fireEvent.click(link);
    expect(nav.view()).toBe("workflows");
    expect(queryByTestId("drilldown-back")).toBeNull();
  });
});
