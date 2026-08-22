import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectRail } from "../../src/shell/ProjectRail";

describe("shell/project-switcher", () => {
  it("renders the selected project and filters the available projects by type", () => {
    const { getByTestId, queryByText } = render(() => (
      <ProjectRail
        selectedProject="locus"
        projects={["locus", "locus-cli", "tapestry"]}
      />
    ));

    expect(getByTestId("selected-project-card").textContent).toContain("locus");
    fireEvent.input(getByTestId("project-switcher-filter"), {
      target: { value: "tape" },
    });
    expect(queryByText("tapestry")).toBeTruthy();
    expect(queryByText("locus-cli")).toBeNull();
  });
});
