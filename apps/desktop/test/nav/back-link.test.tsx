import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { BackLink } from "../../src/nav/BackLink";
import { createNavStore } from "../../src/nav";

describe("nav/back-link", () => {
  it("renders nothing on a view that is not a drill-down", () => {
    const { container } = render(() => (
      <BackLink nav={createNavStore({ view: "workflows" })} />
    ));
    expect(
      container.querySelector('[data-testid="drilldown-back"]'),
    ).toBeNull();
  });

  it("labels the link with the view it goes back to", () => {
    const { getByTestId } = render(() => (
      <BackLink nav={createNavStore({ view: "canvas" })} />
    ));
    const link = getByTestId("drilldown-back");
    expect(link.textContent).toContain("Workflows");
    expect(link.querySelector("use")!.getAttribute("href")).toBe(
      "#ph-arrow-left",
    );
  });

  it("navigates the store to the parent view", () => {
    const nav = createNavStore({ view: "canvas" });
    const { getByTestId } = render(() => <BackLink nav={nav} />);
    fireEvent.click(getByTestId("drilldown-back"));
    expect(nav.view()).toBe("workflows");
  });
});
