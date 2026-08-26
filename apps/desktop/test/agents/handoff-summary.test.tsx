import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import WorkshopFixtureView from "../../src/screens/workshop/WorkshopFixtureView";

describe("agents/handoff-summary", () => {
  it("surfaces the stuck handoff summary in the Agents footer", () => {
    const { getByTestId } = render(() => <WorkshopFixtureView fixture="agents" />);
    const summary = getByTestId("agents-handoff-summary");
    expect(summary.textContent).toContain("Stuck run");
    expect(summary.textContent).toContain("3 iterations without progress");
    expect(summary.textContent).toContain("artifact reference");
  });
});
