import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { usePalette } from "../../src/data/workflow";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("canvas/node-types", () => {
  it("exposes the six executable palette node kinds and no Goal node", () => {
    const { getByTestId, queryByTestId } = render(() => <WorkflowView />);
    for (const node of usePalette())
      expect(getByTestId(`wf-chip-${node.kind}`)).toBeTruthy();
    expect(queryByTestId("wf-chip-goal")).toBeNull();
    expect(getByTestId("wf-node-n-cond").getAttribute("data-node-kind")).toBe(
      "condition",
    );
  });
});
