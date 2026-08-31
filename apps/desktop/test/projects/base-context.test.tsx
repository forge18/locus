import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { configureProjectsStub } from "./provider-stub";

describe("project base context", () => {
  it("renders the live base context and its token budget", async () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <ProjectsView />);

    await waitFor(() =>
      expect(getByTestId("project-base-context-editor").textContent).toContain(
        "# Working in tapestry",
      ),
    );
    expect(getByTestId("project-base-context-budget").textContent).toContain(
      "1500 tokens",
    );
  });

  it("says so when the project has no base context", async () => {
    configureProjectsStub({
      setup: {
        harnessAllowList: ["claude"],
        baseContext: null,
        baseContextTokenBudget: null,
      },
    });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-base-context-editor").textContent).toContain(
        "No base context yet",
      ),
    );
  });

  it("surfaces an IPC failure instead of the old fixture prose", async () => {
    configureProjectsStub({ fail: ["project_setup"] });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-base-context-editor").textContent).toContain(
        "IPC failure for project_setup",
      ),
    );
  });

  it("mounts the shared editor surface from the edit action", async () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-base-context-edit")).toBeTruthy(),
    );
    fireEvent.click(getByTestId("project-base-context-edit"));
    expect(getByTestId("editor-pane")).toBeTruthy();
    expect(getByTestId("editor-surface")).toBeTruthy();
  });

  it("opens the same editor surface in the full-window wrapper", async () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-base-context-edit")).toBeTruthy(),
    );
    fireEvent.click(getByTestId("project-base-context-edit"));
    fireEvent.click(getByTestId("project-base-context-fullscreen"));
    expect(getByTestId("full-window-editor")).toBeTruthy();
    expect(getByTestId("project-editor-overlay")).toBeTruthy();
  });
});
