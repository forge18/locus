import { describe, expect, it } from "vitest";
import { v2PaletteDestinations } from "../../src/nav/LocatorPalette";
import { Desktop_ROUTE_KINDS } from "../../src/nav/desktop-route-kinds";

describe("nav/palette-results", () => {
  it("offers every registered desktop destination before a locator is typed", () => {
    const destinations = v2PaletteDestinations("locus");
    expect(destinations).toHaveLength(Desktop_ROUTE_KINDS.length);
    expect(destinations).toContainEqual({
      label: "Inbox",
      locator: "locus://all/view/inbox",
      section: "Needs you",
    });
    expect(destinations).toContainEqual({
      label: "Plan",
      locator: "locus://locus/view/plan",
      section: "Where you were",
    });
  });
});
