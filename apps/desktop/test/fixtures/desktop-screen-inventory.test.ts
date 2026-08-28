import { describe, expect, it } from "vitest";
import { Desktop_FIXTURE_ROUTES } from "../../src/fixtures/desktop-screen-inventory";

const EXPECTED_SCREENS = [
  "inbox",
  "status",
  "telemetry",
  "mail",
  "projects",
  "plan",
  "sessions",
  "interact",
  "bots",
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

describe("fixtures/desktop-screen-inventory", () => {
  it("registers every delivered desktop screen exactly once", () => {
    expect(Desktop_FIXTURE_ROUTES.map((route) => route.screen)).toEqual(
      EXPECTED_SCREENS,
    );
    expect(new Set(Desktop_FIXTURE_ROUTES.map((route) => route.id)).size).toBe(
      30,
    );
  });

  it("gives every route a stable fixture id, label, scope, and screenshot", () => {
    for (const route of Desktop_FIXTURE_ROUTES) {
      expect(route.id).toMatch(/^[a-z][a-z0-9-]*$/);
      expect(route.label).not.toBe("");
      expect(route.screenshot).toBe(`${route.screen}.png`);
      expect(["all", "app", "project"]).toContain(route.scope);
    }
  });
});
