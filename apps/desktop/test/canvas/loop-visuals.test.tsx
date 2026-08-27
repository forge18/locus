import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("canvas/loop-visuals", () => {
  it("draws the declared loop-back edge and grouping rect", () => {
    const { getByTestId } = render(() => <WorkflowView />);
    expect(
      getByTestId("wf-edge-n-cond-n-build").getAttribute("data-dashed"),
    ).toBe("true");
    expect(getByTestId("wf-loop-group")).toBeTruthy();
    expect(getByTestId("wf-loop-group").textContent).toContain("loop");
  });
});
