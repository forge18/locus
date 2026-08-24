import { describe, expect, it } from "vitest";
import { resolveDesktopLocator } from "../../src/nav/desktop-locator";
import { Desktop_ROUTE_KINDS } from "../../src/nav/desktop-route-kinds";
import { VIEWS } from "../../src/nav/views";
import { desktopLocatorFor, desktopViewFor } from "../../src/shell/Shell";

describe("shell/desktop-route-navigation", () => {
  it("maps every registered desktop locator to an explicit shared surface", () => {
    for (const route of Desktop_ROUTE_KINDS) {
      const locator =
        route.scope === "project"
          ? `locus://locus/view/${route.id}`
          : `locus://${route.scope}/view/${route.id}`;
      expect(desktopViewFor(resolveDesktopLocator(locator))).toBeTruthy();
    }
  });

  it("round-trips every shared surface through its canonical desktop route", () => {
    for (const view of VIEWS) {
      const target = resolveDesktopLocator(desktopLocatorFor(view, "locus"));
      expect(desktopViewFor(target)).toBe(view);
    }
  });
});
