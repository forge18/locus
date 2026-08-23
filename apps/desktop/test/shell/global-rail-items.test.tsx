import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { Desktop_GLOBAL_ROUTE_KINDS } from "../../src/nav/desktop-route-kinds";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/global-rail-items", () => {
  it("renders every global route and the Inbox response badge", () => {
    const { getByTestId } = render(() => (
      <ProjectRail selectedProject="locus" inboxCount={3} />
    ));
    const global = getByTestId("global-rail-routes");

    expect(
      [...global.querySelectorAll("button")].map(
        (item) => item.firstChild?.textContent,
      ),
    ).toEqual(Desktop_GLOBAL_ROUTE_KINDS.map((route) => route.label));
    expect(getByTestId("global-rail-inbox-badge").textContent).toBe("3");
  });
});
