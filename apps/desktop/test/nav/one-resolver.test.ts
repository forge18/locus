import { describe, expect, it } from "vitest";
import {
  destinationDesktop,
  navigateDesktop,
} from "../../src/nav/desktop-navigation";

describe("nav/one-resolver", () => {
  it("uses one boundary to format and resolve rail/palette destinations", () => {
    const locator = destinationDesktop("plan", "locus");
    expect(navigateDesktop(locator)).toEqual({
      route: "plan",
      scope: { kind: "project", project: "locus" },
    });
  });
});
