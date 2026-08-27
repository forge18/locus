import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AnalyticsView } from "../../src/screens/analytics/AnalyticsView";

describe("status/ci-status", () => {
  it("shows normalized CI counts in the at-a-glance status surface", () => {
    const { getByTestId } = render(() => <AnalyticsView />);
    expect(getByTestId("status-ci-status").textContent).toContain("CI checks");
    expect(getByTestId("status-ci-status").textContent).toContain("passed");
    expect(getByTestId("status-ci-status").textContent).toContain("failing");
  });
});
