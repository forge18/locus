import { describe, expect, it } from "vitest";
import { CATEGORIES, activeTabFor, tabsFor } from "../../src/nav";

describe("nav/tab-sets", () => {
  it("gives Telemetry Overview, Telemetry, then Mail", () => {
    expect(tabsFor("telemetry").map((t) => t.label)).toEqual([
      "Overview",
      "Telemetry",
      "Mail",
    ]);
  });

  it("gives Memory its four views in order", () => {
    expect(tabsFor("knowledge").map((t) => t.view)).toEqual([
      "short",
      "memory",
      "artifact",
      "wiki",
    ]);
  });

  it("gives Manage its List surface", () => {
    expect(tabsFor("manage").map((t) => t.view)).toEqual(["sessions"]);
  });

  it("gives non-tab categories no tab set", () => {
    for (const category of [
      "projects",
      "workers",
      "plan",
      "review",
      "settings",
      "pill",
    ] as const) {
      expect(tabsFor(category), category).toEqual([]);
    }
  });

  it("covers every category, so no category is missing a tab set", () => {
    for (const c of CATEGORIES) expect(Array.isArray(tabsFor(c)), c).toBe(true);
  });

  it("lights the tab for a view that has one", () => {
    expect(activeTabFor("telemetry")).toBe("telemetry");
    expect(activeTabFor("wiki")).toBe("wiki");
    expect(activeTabFor("agents")).toBe("agents");
    expect(activeTabFor("plan")).toBe(null);
  });
});
