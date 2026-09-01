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
      locator: "locus://all/view/plan",
      section: "Running now",
    });
  });

  it("moves the current and session destinations into live sections", () => {
    const destinations = v2PaletteDestinations("locus", {
      current: "locus://all/view/status",
      history: ["locus://all/view/plan"],
      sessions: [
        { project: "locus", needsAttention: true },
      ],
    });
    expect(destinations).toContainEqual({
      label: "Status",
      locator: "locus://all/view/status",
      section: "Where you were",
    });
    expect(destinations).toContainEqual({
      label: "Plan",
      locator: "locus://all/view/plan",
      section: "Where you were",
    });
    expect(destinations).toContainEqual({
      label: "Sessions",
      locator: "locus://locus/view/sessions",
      section: "Needs you",
    });
  });
});
