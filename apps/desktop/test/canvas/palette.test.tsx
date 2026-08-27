import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { usePalette } from "../../src/data/workflow";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("canvas/palette", () => {
  it("renders draggable chips for the complete node vocabulary", () => {
    const { getByTestId } = render(() => <WorkflowView />);
    for (const node of usePalette()) {
      expect(
        getByTestId(`wf-chip-${node.kind}`).getAttribute("draggable"),
      ).toBe("true");
    }
  });
});
