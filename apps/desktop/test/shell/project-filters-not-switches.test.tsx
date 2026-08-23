import { createSignal } from "solid-js";
import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { ProjectFilter } from "../../src/shell/ProjectFilter";
import { createNavStore } from "../../src/nav";

const PROJECTS = [
  { id: "p-tapestry", name: "tapestry" },
  { id: "p-loom-db", name: "loom-db" },
  { id: "p-weaver", name: "weaver" },
];

function Harness() {
  const [selected, setSelected] = createSignal<string[]>([]);
  return (
    <>
      <ProjectFilter
        projects={PROJECTS}
        selected={selected()}
        onChange={setSelected}
      />
      <span data-testid="selected">{selected().join(",")}</span>
    </>
  );
}

describe("shell/project-filters-not-switches", () => {
  it("defaults to all projects", () => {
    const { getByTestId } = render(() => <Harness />);
    expect(getByTestId("project-filter-label").textContent).toBe(
      "All projects",
    );
    expect(getByTestId("selected").textContent).toBe("");
  });

  it("shows how many projects there are to filter across", () => {
    const { getByTestId } = render(() => <Harness />);
    expect(getByTestId("project-filter-count").textContent).toBe("3");
  });

  it("narrows to one project without becoming a switcher", async () => {
    const { getByTestId } = render(() => <Harness />);
    getByTestId("context-menu-trigger").dispatchEvent(
      new MouseEvent("contextmenu", { bubbles: true }),
    );
    await waitFor(() => expect(document.querySelector(".menu")).not.toBe(null));
    const weaver = [...document.querySelectorAll(".menu-item")].find(
      (el) => el.textContent === "weaver",
    ) as HTMLElement;
    weaver.dispatchEvent(new PointerEvent("pointerup", { bubbles: true }));
    await waitFor(() =>
      expect(getByTestId("selected").textContent).toBe("p-weaver"),
    );
  });

  it("holds several projects at once, which a switcher could not", () => {
    const [selected, setSelected] = createSignal(["p-tapestry", "p-weaver"]);
    const { getByTestId } = render(() => (
      <ProjectFilter
        projects={PROJECTS}
        selected={selected()}
        onChange={setSelected}
      />
    ));
    expect(getByTestId("project-filter-label").textContent).toBe("2 projects");
  });

  it("never changes the view — filtering is not navigating", () => {
    const nav = createNavStore({ view: "board" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    expect(getByTestId("project-rail")).toBeTruthy();
    expect(nav.view()).toBe("board");
  });
});
