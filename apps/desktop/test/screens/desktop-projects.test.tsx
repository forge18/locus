import { describe, expect, it } from "vitest";
import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { configureProjectsStub } from "../projects/provider-stub";

describe("screens/desktop-projects", () => {
  it("renders the live policy sections from the store", async () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-harnesses").textContent).toContain("claude"),
    );
    expect(getByTestId("project-settings")).toBeTruthy();
    expect(getByTestId("project-repos")).toBeTruthy();
    expect(getByTestId("project-base-context")).toBeTruthy();
  });

  it("renders every harness the project policy allows", async () => {
    configureProjectsStub();
    const { getByTestId, getByText } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-harnesses").textContent).toContain("codex"),
    );
    expect(getByText("claude").textContent).toBe("claude");
  });

  it("switches to the shared scoped analytics projection", async () => {
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
    expect(getByTestId("analytics-stat-cards")).toBeTruthy();
    expect(getByTestId("analytics-breakdown")).toBeTruthy();
  });
});
