import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";

describe("project analytics", () => {
  it("renders the shared scoped analytics projection", () => {
    const { getByTestId } = render(() => <ProjectsView />);
    fireEvent.click(getByTestId("project-tab-analytics"));
    expect(getByTestId("analytics").getAttribute("data-scope")).toBe(
      "tapestry",
    );
    for (const measure of ["spend", "tokens", "cache", "runs"]) {
      expect(getByTestId(`analytics-stat-${measure}`)).toBeTruthy();
    }
  });
});
