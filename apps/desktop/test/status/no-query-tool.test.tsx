import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AnalyticsView } from "../../src/screens/analytics/AnalyticsView";

describe("status/no-query-tool", () => {
  it("keeps search and facet controls out of Status", () => {
    const { getByTestId, queryByTestId, queryByPlaceholderText } = render(
      () => <AnalyticsView />,
    );
    expect(getByTestId("status-metrics")).toBeTruthy();
    expect(queryByTestId("analytics-telemetry")).toBeNull();
    expect(queryByTestId("analytics-search")).toBeNull();
    expect(
      queryByPlaceholderText("BM25 search over the normalized event log"),
    ).toBeNull();
  });
});
