import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { createNavStore } from "../../src/nav";

const mount = () => {
  const nav = createNavStore({ view: "telemetry" });
  return {
    nav,
    ...render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    )),
  };
};

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("nav/current-view-context", () => {
  it("shows the owning context for title-bar routes", () => {
    const { nav, getByTestId } = mount();
    nav.go("runs");
    expect(getByTestId("title-category").textContent).toBe("Inbox");
    expect(getByTestId("title-view").textContent).toBe("runs");
    nav.go("artifact");
    expect(getByTestId("title-category").textContent).toBe("Memory");
    expect(getByTestId("title-view").textContent).toBe("artifact");
  });

  it("keeps the shared rail in place when switching current views", () => {
    const { nav, getByTestId } = mount();
    const rail = getByTestId("project-rail");
    nav.go("runs");
    expect(getByTestId("project-rail")).toBe(rail);
  });
});
