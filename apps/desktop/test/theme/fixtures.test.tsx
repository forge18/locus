import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { AnalyticsView } from "../../src/screens/analytics/AnalyticsView";
import { createNavStore } from "../../src/nav";
import { INSTALLED_THEMES } from "../../src/styles/theme";

afterEach(cleanup);

describe("theme/fixtures", () => {
  for (const theme of INSTALLED_THEMES) {
    it(`renders the desktop fixture inventory in ${theme}`, () => {
      const inbox = render(() => (
        <div data-theme={theme}>
          <InboxView nav={createNavStore({ view: "inbox" })} />
        </div>
      ));
      expect(
        inbox.getByTestId("inbox").closest(`[data-theme="${theme}"]`),
      ).toBeTruthy();
      inbox.unmount();

      const analytics = render(() => (
        <div data-theme={theme}>
          <AnalyticsView />
        </div>
      ));
      expect(
        analytics.getByTestId("analytics").closest(`[data-theme="${theme}"]`),
      ).toBeTruthy();
    });
  }
});
