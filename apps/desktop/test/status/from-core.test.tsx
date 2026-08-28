import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AT_A_GLANCE_METRICS } from "../../src/data/analytics";
import { AnalyticsView } from "../../src/screens/analytics/AnalyticsView";

describe("status/from-core", () => {
  it("renders the at-a-glance metrics from the analytics projection", () => {
    const { getByTestId } = render(() => <AnalyticsView />);
    const rail = getByTestId("status-metrics");
    for (const metric of AT_A_GLANCE_METRICS) {
      expect(
        rail.querySelector(`[data-metric="${metric.id}"]`)?.textContent,
      ).toContain(metric.value);
    }
  });
});
