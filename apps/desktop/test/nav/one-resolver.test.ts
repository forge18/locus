import { describe, expect, it } from "vitest";
import { destinationV2, navigateV2 } from "../../src/nav/v2-navigation";

describe("nav/one-resolver", () => {
  it("uses one boundary to format and resolve rail/palette destinations", () => {
    const locator = destinationV2("develop", "locus");
    expect(navigateV2(locator)).toEqual({
      route: "develop",
      scope: { kind: "project", project: "locus" },
    });
  });
});
