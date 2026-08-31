import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ProjectsView } from "../../src/screens/projects/ProjectsView";
import { LOOM, TAPESTRY, configureProjectsStub } from "./provider-stub";
import { fetchProjects } from "../../src/data/core";

describe("project mutations (slice 5)", () => {
  it("saves the edited base context through the registered command", async () => {
    const stub = configureProjectsStub({
      mutations: {
        project_base_context_set: {
          harnessAllowList: ["claude", "codex"],
          baseContext: "# Edited",
          baseContextTokenBudget: 1500,
        },
      },
    });
    const { getByTestId } = render(() => <ProjectsView />);
    await waitFor(() =>
      expect(getByTestId("project-base-context-edit")).toBeTruthy(),
    );

    fireEvent.click(getByTestId("project-base-context-edit"));
    fireEvent.click(getByTestId("project-base-context-save"));

    await waitFor(() =>
      expect(
        stub.calls.some(
          (call) =>
            call.command === "project_base_context_set" &&
            (call.args?.projectId as string) === LOOM,
        ),
      ).toBe(true),
    );
    // The saved state replaces the panel's envelope, so the store value shows.
    await waitFor(() =>
      expect(getByTestId("project-base-context-editor").textContent).toContain(
        "# Edited",
      ),
    );
  });

  it("archives through the registered command and refreshes", async () => {
    const stub = configureProjectsStub({
      mutations: { project_archive_set: { archived: true } },
    });
    const { getByText } = render(() => <ProjectsView />);
    await waitFor(() => expect(getByText("#tapestry")).toBeTruthy());

    fireEvent.click(getByText("Archive"));

    await waitFor(() =>
      expect(
        stub.calls.some(
          (call) =>
            call.command === "project_archive_set" &&
            (call.args?.archived as boolean) === true,
        ),
      ).toBe(true),
    );
  });

  it("renames through the registered command and shows the new name", async () => {
    const stub = configureProjectsStub({
      mutations: { project_rename: { id: TAPESTRY, name: "weaver" } },
    });
    const { getByText, getByTestId } = render(() => <ProjectsView />);
    await waitFor(() => expect(getByText("#tapestry")).toBeTruthy());

    fireEvent.click(getByText("Rename"));
    const input = getByTestId("project-rename-input");
    fireEvent.input(input, { target: { value: "weaver" } });
    fireEvent.click(getByText("Save name"));

    await waitFor(() =>
      expect(
        stub.calls.some(
          (call) =>
            call.command === "project_rename" &&
            (call.args?.name as string) === "weaver",
        ),
      ).toBe(true),
    );
    // The refreshed project list carries the renamed row.
    const after = await fetchProjects();
    await waitFor(() =>
      expect(
        getByTestId("project-state-list").textContent,
        `stub=${JSON.stringify(after.status === "ready" ? after.data : after.status)} calls=${stub.calls.length}`,
      ).toContain("weaver"),
    );
  });
});
