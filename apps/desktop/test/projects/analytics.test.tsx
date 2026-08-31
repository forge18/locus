import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { configureProjectsStub } from "./provider-stub";
// The assertion uses the project NAME because AnalyticsView scopes by name.

describe("project analytics", () => {
  it("renders the shared scoped analytics projection", async () => {
    configureProjectsStub();
    const { getByTestId, getAllByText } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getAllByText("#tapestry").length).toBeGreaterThan(0),
    );
    fireEvent.click(getAllByText("#tapestry")[0]);
    fireEvent.click(getByTestId("project-tab-analytics"));
    expect(getByTestId("analytics").getAttribute("data-scope")).toBe(
      "tapestry",
    );
    for (const measure of ["spend", "tokens", "cache", "runs"]) {
      expect(getByTestId(`analytics-stat-${measure}`)).toBeTruthy();
    }
  });
});
