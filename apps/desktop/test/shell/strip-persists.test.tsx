import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { VIEWS, createNavStore } from "../../src/nav";

describe("shell/shared-surfaces", () => {
  it("keeps the titlebar, project rail, and running summary through every view", () => {
    const nav = createNavStore({ view: "inbox" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    const titlebar = getByTestId("app-titlebar");
    const rail = getByTestId("project-rail");
    const running = getByTestId("running-pill");

    for (const view of VIEWS) {
      nav.go(view);
      expect(getByTestId("app-titlebar"), view).toBe(titlebar);
      expect(getByTestId("project-rail"), view).toBe(rail);
      expect(getByTestId("running-pill"), view).toBe(running);
      expect(getByTestId("title-view").textContent, view).toBe(view);
    }
  });

  it("updates the visible category while retaining the shared surfaces", () => {
    const nav = createNavStore({ view: "board" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    expect(getByTestId("title-category").textContent).toBe("Automate");
    nav.go("telemetry");
    expect(getByTestId("title-category").textContent).toBe("Review");
    expect(getByTestId("project-rail")).toBeTruthy();
  });
});
