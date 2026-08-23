import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/rail-active", () => {
  it("makes the first global route reachable by keyboard", () => {
    const { getByTestId, getByText } = render(() => (
      <ProjectRail selectedProject="tapestry" />
    ));
    const routes = getByTestId("global-rail-routes");
    expect(getByText("Inbox").getAttribute("tabindex")).toBe("0");
    expect(routes.querySelectorAll('button[tabindex="0"]')).toHaveLength(1);
  });

  it("moves the roving focus target through global routes", () => {
    const { getByTestId, getByText } = render(() => (
      <ProjectRail selectedProject="tapestry" />
    ));
    fireEvent.keyDown(getByTestId("global-rail-routes"), { key: "ArrowDown" });
    expect(getByText("Inbox").getAttribute("tabindex")).toBe("-1");
    expect(getByText("Dashboard").getAttribute("tabindex")).toBe("0");
  });

  it("reports dispatch state without relying on a visual color alias", () => {
    const { getByTestId } = render(() => (
      <ProjectRail selectedProject="tapestry" dispatchState="blocked" />
    ));
    expect(getByTestId("dispatch-dot").getAttribute("data-state")).toBe(
      "blocked",
    );
  });
});
