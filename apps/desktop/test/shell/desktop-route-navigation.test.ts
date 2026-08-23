import { describe, expect, it } from "vitest";
import { resolveDesktopLocator } from "../../src/nav/desktop-locator";
import { Desktop_ROUTE_KINDS } from "../../src/nav/desktop-route-kinds";
import { desktopViewFor } from "../../src/shell/Shell";

describe("shell/desktop-route-navigation", () => {
  it("maps every registered desktop locator to an explicit shared surface", () => {
    for (const route of Desktop_ROUTE_KINDS) {
      const locator = route.scope === "global"
        ? `locus://global/${route.id}`
        : `locus://project/locus/${route.id}`;
      expect(desktopViewFor(resolveDesktopLocator(locator))).toBeTruthy();
    }
  });
});
