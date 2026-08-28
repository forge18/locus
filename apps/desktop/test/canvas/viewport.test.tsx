import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("canvas/viewport", () => {
  it("mounts pan and zoom controls with a zoom pill", () => {
    const { getByTestId } = render(() => <WorkflowView />);
    expect(
      getByTestId("wf-solid-flow").querySelector(".solid-flow__pane"),
    ).toBeTruthy();
    expect(getByTestId("wf-zoom").textContent).toBe("100%");
  });
});
