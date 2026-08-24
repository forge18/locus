import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { PlanView } from "../../src/screens/plan/PlanView";

describe("planning conversation", () => {
  it("renders stage progress and an ACP live line", () => {
    const { getByTestId } = render(() => <PlanView />);
    expect(getByTestId("plan-stage-progress").textContent).toContain(
      "Step 3 of 7",
    );
    expect(getByTestId("plan-live").textContent).not.toBe("");
  });
});
