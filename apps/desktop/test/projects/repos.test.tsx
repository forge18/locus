import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { TAPESTRY, configureProjectsStub } from "./provider-stub";

describe("project repos", () => {
  it("renders the selected project's repos with their working-copy paths", async () => {
    configureProjectsStub();
    const { getAllByTestId, getByTestId, getByText, getAllByText } = render(
      () => <ProjectsView />,
    );

    // loom-db sorts first; select tapestry, which has two repos.
    await waitFor(() => expect(getByText("#tapestry")).toBeTruthy());
    fireEvent.click(getAllByText("#tapestry")[0]);

    await waitFor(() =>
      expect(getAllByTestId("project-repo-row").length).toBe(2),
    );
    const rows = getAllByTestId("project-repo-row");
    expect(rows[0].textContent).toContain("core");
    expect(rows[0].textContent).toContain("/checkouts/tapestry-core");
    expect(rows[1].textContent).toContain("desktop");
    expect(rows[1].textContent).toContain("/checkouts/tapestry-desktop");
    // Another project's repo never leaks into this list.
    expect(getByTestId("project-repos").textContent).not.toContain(
      "/checkouts/loom",
    );
  });

  it("renders an empty project honestly", async () => {
    configureProjectsStub({
      projects: [{ id: TAPESTRY, name: "tapestry" }],
      repos: [],
    });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-repos").textContent).toContain(
        "No repos in this project yet",
      ),
    );
  });

  it("surfaces an IPC failure instead of an empty success", async () => {
    configureProjectsStub({ fail: ["repos_list"] });
    const { getByTestId, getByText } = render(() => <ProjectsView />);
    await waitFor(() => expect(getByText("#tapestry")).toBeTruthy());
    await waitFor(() =>
      expect(getByTestId("project-repos").textContent).toContain(
        "IPC failure for repos_list",
      ),
    );
  });
});
