import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { DesktopDashboardView, DesktopInboxView } from "../../src/screens/desktop-dashboard";
import { INSTALLED_THEMES } from "../../src/styles/theme";

afterEach(cleanup);

describe("theme/fixtures", () => {
  for (const theme of INSTALLED_THEMES) {
    it(`renders the desktop fixture inventory in ${theme}`, () => {
      const inbox = render(() => (
        <div data-theme={theme}>
          <DesktopInboxView />
        </div>
      ));
      expect(
        inbox.getByTestId("desktop-inbox").closest(`[data-theme="${theme}"]`),
      ).toBeTruthy();
      inbox.unmount();

      const dashboard = render(() => (
        <div data-theme={theme}>
          <DesktopDashboardView />
        </div>
      ));
      expect(
        dashboard
          .getByTestId("desktop-dashboard")
          .closest(`[data-theme="${theme}"]`),
      ).toBeTruthy();
    });
  }
});
