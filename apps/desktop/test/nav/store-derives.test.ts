import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";

describe("nav/store-derives", () => {
  it("derives the category from the view", () => {
    const nav = createNavStore();
    expect(nav.category()).toBe("pill");
    nav.go("runs");
    expect(nav.category()).toBe("pill");
  });

  it("derives the category label the rail and tab bar show", () => {
    const nav = createNavStore({ view: "status" });
    expect(nav.categoryLabel()).toBe("Telemetry");
    nav.go("canvas");
    expect(nav.categoryLabel()).toBe("Extensions");
  });

  it("derives the locator, and the path form the bars render", () => {
    const nav = createNavStore({ project: "weaver", view: "plan" });
    expect(nav.locator()).toBe("locus://all/view/plan");
    expect(nav.locatorPath()).toBe("all/view/plan");
  });

  it("derives the visible tab set", () => {
    const nav = createNavStore({ view: "status" });
    expect(nav.tabs().map((t) => t.label)).toEqual([
      "Overview",
      "Telemetry",
      "Mail",
    ]);
    nav.go("memory");
    expect(nav.tabs().map((t) => t.label)).toEqual([
      "Short-term",
      "Long-term",
      "Artifacts",
      "Wiki",
    ]);
    nav.go("plan");
    expect(nav.tabs()).toEqual([]);
  });

  it("normalizes params through the grammar, so nothing survives that a locator cannot carry", () => {
    const nav = createNavStore();
    nav.go("sessions", { project: "tapestry", sessionId: "8f21" });
    expect(nav.params()).toEqual({ project: "tapestry", sessionId: "8f21" });
    // Moving on drops the id and the page-owned project filter.
    nav.go("plan");
    expect(nav.params()).toEqual({});
  });

  it("does not carry a project filter across page-owned routes", () => {
    const nav = createNavStore({ project: "weaver" });
    nav.go("sessions");
    nav.go("plan");
    expect(nav.params()).toEqual({});
  });

  it("preserves explicit global scope without restoring the last project", () => {
    const nav = createNavStore({ project: "weaver" });
    nav.go("status", { project: undefined });
    expect(nav.params()).toEqual({});
    expect(nav.locator()).toBe("locus://all/view/status");
  });
});
