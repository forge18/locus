import { describe, expect, it } from "vitest";
import { DESKTOP_ROUTES } from "../../src/nav/desktop-screen-inventory";
import {
  destinationDesktop,
  navigateDesktop,
} from "../../src/nav/desktop-navigation";
import { CATEGORIES, RAIL_ITEMS, VIEWS, categoryOf } from "../../src/nav";

describe("planning-workspace navigation inventory", () => {
  it("registers the current routes and page-owned categories", () => {
    expect(VIEWS).toHaveLength(29);
    expect(DESKTOP_ROUTES).toHaveLength(29);
    expect([...CATEGORIES]).toEqual([
      "projects",
      "workers",
      "telemetry",
      "plan",
      "manage",
      "review",
      "extensions",
      "plugins",
      "knowledge",
      "settings",
    ]);
    expect(RAIL_ITEMS.map((item) => item.firstView)).toEqual([
      "projects",
      "workers",
      "telemetry",
      "plan",
      "sessions",
      "qa",
      "agents",
      "cli",
      "short",
      "settings",
    ]);
  });

  it("routes every category landing view without retired Interact", () => {
    expect(RAIL_ITEMS.map((item) => item.label)).toEqual([
      "Projects",
      "Workers",
      "Telemetry",
      "Plan",
      "Manage",
      "Review",
      "Extensions",
      "Plugins",
      "Knowledge",
      "Settings",
    ]);
    expect(
      DESKTOP_ROUTES.some((route) =>
        ["Develop", "Automate", "Dashboard", "Interact"].includes(route.label),
      ),
    ).toBe(false);
    for (const route of DESKTOP_ROUTES) {
      const locator = destinationDesktop(
        route.id,
        route.scope === "project" ? "tapestry" : undefined,
      );
      expect(navigateDesktop(locator).route).toBe(route.id);
      expect(categoryOf(route.id)).toBe(route.category);
    }
  });

  it("uses page-owned all, app, and project locators", () => {
    expect(destinationDesktop("projects")).toBe("locus://all/view/projects");
    expect(destinationDesktop("workers")).toBe("locus://all/view/workers");
    expect(destinationDesktop("status")).toBe("locus://all/view/status");
    expect(destinationDesktop("settings")).toBe("locus://app/view/settings");
    expect(navigateDesktop("locus://all/view/qa")).toEqual({
      route: "qa",
      scope: { kind: "all" },
    });
    expect(navigateDesktop("locus://all/view/inbox")).toEqual({
      route: "inbox",
      scope: { kind: "all" },
    });
    expect(navigateDesktop("locus://app/view/workflows")).toEqual({
      route: "workflows",
      scope: { kind: "app" },
    });
    expect(navigateDesktop("locus://tapestry/workers/keeper")).toEqual({
      route: "workers",
      scope: { kind: "project", project: "tapestry" },
      botId: "keeper",
    });
  });
});
