import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import {
  LOOM,
  SEED_PROJECTS,
  TAPESTRY,
  configureProjectsStub,
} from "./provider-stub";

describe("project list", () => {
  it("renders the real project list once it loads", async () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-state-list").textContent).toContain("tapestry"),
    );
    expect(getByTestId("project-state-list").textContent).toContain("loom-db");
  });

  it("selects a project on click and retargets the detail header", async () => {
    configureProjectsStub();
    const { getByText, getAllByText, getByTestId } = render(() => <ProjectsView />);

    // loom-db sorts first, so it is selected by default; click tapestry.
    await waitFor(() => expect(getByText("#tapestry")).toBeTruthy());
    fireEvent.click(getAllByText("#tapestry")[0]);

    await waitFor(() =>
      expect(getByTestId("projects-view").textContent).toContain(
        "locus://tapestry",
      ),
    );
  });

  it("pins the loading state while queries are in flight", () => {
    configureProjectsStub({ hang: true });
    const { getByTestId } = render(() => <ProjectsView />);
    expect(getByTestId("project-state-list").textContent).toContain(
      "Loading projects…",
    );
  });

  it("renders an empty list honestly", async () => {
    configureProjectsStub({ projects: [] });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-state-list").textContent).toContain(
        "No projects yet",
      ),
    );
  });

  it("surfaces an IPC failure with a retry instead of a fixture", async () => {
    configureProjectsStub({ fail: true });
    const { getByTestId, getByText } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-state-list").textContent).toContain(
        "IPC failure for projects_list",
      ),
    );
    fireEvent.click(getByText("Retry"));
    await waitFor(() =>
      expect(getByTestId("project-state-list").textContent).toContain(
        "IPC failure for projects_list",
      ),
    );
  });

  it("the seeded ids match the host fixtures", () => {
    expect(SEED_PROJECTS.map((project) => project.id)).toEqual([
      LOOM,
      TAPESTRY,
    ]);
  });
});
