import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";

describe("palette/history", () => {
  it("traverses back and forward through locator history", () => {
    const nav = createNavStore({ view: "inbox" });
    nav.go("plan", { project: "tapestry" });
    nav.go("wiki", { project: "tapestry" });
    nav.back();
    expect(nav.view()).toBe("plan");
    nav.forward();
    expect(nav.view()).toBe("wiki");
  });
});
