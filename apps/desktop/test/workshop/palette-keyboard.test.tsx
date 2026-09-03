import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("workshop/palette-keyboard", () => {
  it("keeps chips draggable and gives them a real button", () => {
    const { getByTestId } = render(() => <WorkflowView />);
    const chip = getByTestId("wf-chip-task");
    expect(chip.tagName).toBe("BUTTON");
    expect(chip.getAttribute("draggable")).toBe("true");
  });

  it("places the node when a chip is activated from the keyboard", () => {
    const view = render(() => <WorkflowView />);
    const count = () =>
      view.getByTestId("wf-canvas").querySelectorAll(".wf-flow-node").length;
    const initial = count();
    fireEvent.click(view.getByTestId("wf-chip-task"));
    expect(count()).toBe(initial + 1);
  });
});
