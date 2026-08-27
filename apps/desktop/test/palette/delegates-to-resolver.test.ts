import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("palette/delegates-to-resolver", () => {
  it("keeps navigation parsing in desktop-navigation", () => {
    const source = readFileSync(resolve("src/nav/LocatorPalette.tsx"), "utf8");
    expect(source).toContain("navigateDesktop");
    expect(source).not.toMatch(/parse\(/);
  });
});
