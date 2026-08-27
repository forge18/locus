import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";

describe("canvas/inspector", () => {
  it("shows the compiled deterministic condition expression", () => {
    const { getByTestId } = render(() => <WorkflowView />);
    expect(getByTestId("wf-inspector")).toBeTruthy();
    expect(getByTestId("wf-compiled-expr").textContent).toContain(
      "verify.passed",
    );
    expect(getByTestId("wf-compiled-note").textContent).toContain(
      "evaluable in the core",
    );
  });
});
