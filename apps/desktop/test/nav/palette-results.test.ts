import { describe, expect, it } from "vitest";
import { v2PaletteDestinations } from "../../src/nav/LocatorPalette";

describe("nav/palette-results", () => {
  it("offers recognizable v2 destinations before a locator is typed", () => {
    expect(v2PaletteDestinations()).toContainEqual({
      label: "Inbox",
      locator: "locus://global/inbox",
    });
  });
});
