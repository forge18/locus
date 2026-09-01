import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";
import { createNavStore } from "../../src/nav";

describe("nav/project-is-filter", () => {
  it("does not own a project selector", () => {
    const { queryByTestId } = render(() => (
      <ProjectRail selectedProject="tapestry" projects={["tapestry", "weaver"]} />
    ));
    expect(queryByTestId("project-switcher-option-tapestry")).toBeNull();
    expect(queryByTestId("project-switcher-results")).toBeNull();
  });

  it("uses an all-project locator for page-owned routes", () => {
    const nav = createNavStore({ project: "weaver", view: "sessions" });
    expect(nav.locator()).toBe("locus://all/view/sessions");
    expect(nav.params()).toEqual({});
  });
});
