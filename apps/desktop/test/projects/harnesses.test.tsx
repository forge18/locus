import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { configureProjectsStub } from "./provider-stub";

describe("project harnesses", () => {
  it("renders the stored allow list", async () => {
    configureProjectsStub();
    const { getByTestId, getByText } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-harnesses").textContent).toContain("claude"),
    );
    expect(getByText("codex").textContent).toBe("codex");
    expect(getByTestId("project-router-summary").textContent).toContain(
      "agent default",
    );
  });

  it("says so when no harness policy is stored", async () => {
    configureProjectsStub({
      setup: {
        harnessAllowList: [],
        baseContext: null,
        baseContextTokenBudget: null,
      },
    });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-harnesses").textContent).toContain(
        "No harness policy is stored",
      ),
    );
  });

  it("surfaces an IPC failure instead of an invented policy", async () => {
    configureProjectsStub({ fail: ["project_setup"] });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-harnesses").textContent).toContain(
        "IPC failure for project_setup",
      ),
    );
  });
});
