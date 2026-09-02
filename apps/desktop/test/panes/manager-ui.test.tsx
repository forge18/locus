import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { PaneManager } from "../../src/panes/PaneManager";
import type { Pane } from "../../src/panes/manager";

const initial: Pane = {
  id: "one",
  kind: "agent",
  runId: "run-one",
  focusedAt: 1,
};

function mount(
  onDetach: (pane: Pane) => Promise<unknown> | unknown = () => undefined,
) {
  let created = 0;
  return render(() => (
    <PaneManager
      initialPane={initial}
      createPane={(source) => ({
        ...source,
        id: created++ === 0 ? "two" : "three",
        focusedAt: 2,
      })}
      onDetach={onDetach}
      renderPane={(pane) => (
        <span data-testid={`pane-content-${pane.id}`}>{pane.id}</span>
      )}
    />
  ));
}

describe("PaneManager UI", () => {
  it("splits, minimizes, promotes, and closes panes without losing identity", async () => {
    const view = mount();

    await fireEvent.click(view.getByTestId("pane-split-one"));
    expect(view.getByTestId("pane-content-two")).toBeTruthy();
    expect(view.getByTestId("pane-focused-count").textContent).toContain("2");

    await fireEvent.click(view.getByTestId("pane-split-two"));
    expect(view.getByTestId("pane-content-three")).toBeTruthy();
    expect(view.getByTestId("pane-focused-count").textContent).toContain("3");

    await fireEvent.click(view.getByTestId("pane-minimize-one"));
    expect(view.queryByTestId("pane-content-one")).toBeNull();
    expect(view.getByTestId("pane-promote-one")).toBeTruthy();

    await fireEvent.click(view.getByTestId("pane-promote-one"));
    expect(view.getByTestId("pane-content-one")).toBeTruthy();
    expect(view.getByTestId("pane-strip-count").textContent).toContain("0");

    await fireEvent.click(view.getByTestId("pane-close-one"));
    expect(view.queryByTestId("pane-content-one")).toBeNull();
    expect(view.getByTestId("pane-content-two")).toBeTruthy();
  });

  it("detaches a pane through the host callback with its run identity", async () => {
    const detached: Pane[] = [];
    const view = mount((pane) => detached.push(pane));

    await fireEvent.click(view.getByTestId("pane-detach-one"));

    expect(detached).toEqual([initial]);
  });
});
