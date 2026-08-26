import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";

describe("project base context", () => {
  it("renders editable context with its token budget meter", () => {
    const { getByTestId } = render(() => <ProjectsView />);
    expect(getByTestId("project-base-context-editor").textContent).toContain(
      "base.md",
    );
    expect(getByTestId("project-base-context-budget").textContent).toContain(
      "1,500 tokens",
    );
  });

  it("mounts the shared editor surface from the edit action", () => {
    const { getByTestId } = render(() => <ProjectsView />);
    getByTestId("project-base-context-edit").click();
    expect(getByTestId("editor-pane")).toBeTruthy();
    expect(getByTestId("editor-surface")).toBeTruthy();
  });

  it("opens the same editor surface in the full-window wrapper", () => {
    const { getByTestId } = render(() => <ProjectsView />);
    getByTestId("project-base-context-edit").click();
    getByTestId("project-base-context-fullscreen").click();
    expect(getByTestId("full-window-editor")).toBeTruthy();
    expect(getByTestId("project-editor-overlay")).toBeTruthy();
  });
});
