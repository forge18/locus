import { describe, expect, it } from "vitest";
import { DESKTOP_ROUTES } from "../../src/nav/desktop-screen-inventory";

const EXPECTED_SCREENS = [
  "inbox",
  "status",
  "telemetry",
  "mail",
  "projects",
  "workers",
  "plan",
  "sessions",
  "qa",
  "autorun",
  "schedule",
  "runs",
  "short",
  "memory",
  "artifact",
  "wiki",
  "settings",
  "agents",
  "cli",
  "commands",
  "harnesses",
  "hooks",
  "linters",
  "styles",
  "providers",
  "rules",
  "skills",
  "canvas",
  "workflows",
] as const;

describe("nav/desktop-screen-inventory", () => {
  it("registers every delivered desktop screen exactly once", () => {
    expect(DESKTOP_ROUTES.map((route) => route.screen)).toEqual(
      EXPECTED_SCREENS,
    );
    expect(new Set(DESKTOP_ROUTES.map((route) => route.id)).size).toBe(29);
  });

  it("gives every route a stable route id, label, scope, and screenshot", () => {
    for (const route of DESKTOP_ROUTES) {
      expect(route.id).toMatch(/^[a-z][a-z0-9-]*$/);
      expect(route.label).not.toBe("");
      expect(route.screenshot).toBe(`${route.screen}.png`);
      expect(["all", "app", "project"]).toContain(route.scope);
    }
  });
});
