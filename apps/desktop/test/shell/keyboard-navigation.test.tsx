import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/keyboard-navigation", () => {
  it("uses one roving tab stop for global rail buttons", () => {
    const { getByTestId, getByText } = render(() => (
      <ProjectRail selectedProject="locus" />
    ));
    const routes = getByTestId("global-rail-routes");
    const inbox = getByText("Inbox");
    const dashboard = getByText("Dashboard");

    expect(inbox.getAttribute("tabindex")).toBe("0");
    fireEvent.keyDown(routes, { key: "ArrowDown" });
    expect(dashboard.getAttribute("tabindex")).toBe("0");
    expect(inbox.getAttribute("tabindex")).toBe("-1");
  });
});
