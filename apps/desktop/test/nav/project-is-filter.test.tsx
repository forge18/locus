import { describe, expect, it } from "vitest";
import { fireEvent, render } from "@solidjs/testing-library";
import { ProjectRail } from "../../src/shell/ProjectRail";
import { createNavStore } from "../../src/nav";

const PROJECTS = ["tapestry", "weaver"];

describe("nav/project-is-filter", () => {
  it("shows the selected project and its available matches", () => {
    const { getByTestId } = render(() => (
      <ProjectRail selectedProject="tapestry" projects={PROJECTS} />
    ));
    expect(
      getByTestId("project-switcher-option-tapestry").textContent,
    ).toContain("tapestry");
    expect(getByTestId("project-switcher-results").children).toHaveLength(2);
  });

  it("filters project choices without changing the current route", () => {
    const nav = createNavStore({ view: "sessions" });
    const { getByTestId } = render(() => (
      <ProjectRail selectedProject="tapestry" projects={PROJECTS} />
    ));
    fireEvent.input(getByTestId("project-switcher-filter"), {
      target: { value: "weaver" },
    });
    expect(getByTestId("project-switcher-results").textContent).toBe("weaver");
    expect(nav.view()).toBe("sessions");
  });

  it("keeps an explicitly addressed project in its locator", () => {
    const nav = createNavStore({ project: "weaver", view: "sessions" });
    expect(nav.locator()).toBe("locus://weaver/view/sessions");
  });
});
