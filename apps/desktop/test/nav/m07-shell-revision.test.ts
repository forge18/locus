import { describe, expect, it } from "vitest";
import { DESKTOP_ROUTES } from "../../src/nav/desktop-screen-inventory";
import {
  destinationDesktop,
  navigateDesktop,
} from "../../src/nav/desktop-navigation";
import { CATEGORIES, RAIL_ITEMS, VIEWS, categoryOf } from "../../src/nav";

describe("M0.7 shell navigation inventory", () => {
  it("registers the current 30 views and ten rail categories", () => {
    expect(VIEWS).toHaveLength(30);
    expect(DESKTOP_ROUTES).toHaveLength(30);
    expect([...CATEGORIES]).toEqual([
      "setup",
      "plan",
      "manage",
      "interact",
      "bots",
      "review",
      "analytics",
      "memory",
      "settings",
      "workshop",
    ]);
    expect(RAIL_ITEMS.map((item) => item.firstView)).toEqual([
      "projects",
      "plan",
      "sessions",
      "interact",
      "bots",
      "qa",
      "status",
      "short",
      "settings",
      "agents",
    ]);
  });

  it("routes every category landing view without retired rail vocabulary", () => {
    expect(RAIL_ITEMS.map((item) => item.label)).toEqual([
      "Setup",
      "Plan",
      "Manage",
      "Interact",
      "Bots",
      "Review",
      "Analytics",
      "Memory",
      "Settings",
      "Workshop",
    ]);
    expect(
      DESKTOP_ROUTES.some((route) =>
        ["Develop", "Automate", "Dashboard"].includes(route.label),
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

  it("uses the canonical project, all, and app view locators", () => {
    expect(destinationDesktop("projects", "tapestry")).toBe(
      "locus://tapestry/view/projects",
    );
    expect(destinationDesktop("status")).toBe("locus://all/view/status");
    expect(destinationDesktop("settings")).toBe("locus://app/view/settings");
    expect(navigateDesktop("locus://tapestry/view/plan")).toEqual({
      route: "plan",
      scope: { kind: "project", project: "tapestry" },
    });
    expect(navigateDesktop("locus://all/view/inbox")).toEqual({
      route: "inbox",
      scope: { kind: "all" },
    });
    expect(navigateDesktop("locus://app/view/workflows")).toEqual({
      route: "workflows",
      scope: { kind: "app" },
    });
  });
});
