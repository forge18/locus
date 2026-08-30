import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { Shell } from "../../src/shell/Shell";
import { configureProjectsStub } from "../projects/provider-stub";

describe("shell/project-switcher", () => {
  it("offers the live project list in the rail switcher", async () => {
    configureProjectsStub();
    const nav = createNavStore();
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <div />
      </Shell>
    ));

    // The fallback rail shows only the selected project; loom-db proves the
    // live project list has landed.
    await waitFor(() =>
      expect(getByTestId("project-switcher-option-loom-db")).toBeTruthy(),
    );
    expect(getByTestId("project-switcher-option-loom-db")).toBeTruthy();
  });

  it("switching projects updates the canonical locator", async () => {
    configureProjectsStub();
    const nav = createNavStore();
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <div />
      </Shell>
    ));
    // The fallback rail shows only the selected project; loom-db proves the
    // live project list has landed.
    await waitFor(() =>
      expect(getByTestId("project-switcher-option-loom-db")).toBeTruthy(),
    );

    fireEvent.click(getByTestId("project-switcher-option-loom-db"));

    await waitFor(() => expect(nav.params().project).toBe("loom-db"));
    expect(nav.locator()).toContain("loom-db");
  });

  it("marks the current project as selected", async () => {
    configureProjectsStub();
    const nav = createNavStore();
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <div />
      </Shell>
    ));
    // The fallback rail shows only the selected project; loom-db proves the
    // live project list has landed.
    await waitFor(() =>
      expect(getByTestId("project-switcher-option-loom-db")).toBeTruthy(),
    );

    // The default nav project is tapestry: it is the selected option.
    expect(
      getByTestId("project-switcher-option-tapestry").getAttribute(
        "data-selected-project",
      ),
    ).toBe("true");
  });
});
