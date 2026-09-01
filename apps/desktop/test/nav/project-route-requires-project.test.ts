import { describe, expect, it } from "vitest";
import {
  Desktop_ALL_ROUTE_KINDS,
  Desktop_PROJECT_ROUTE_KINDS,
} from "../../src/nav/desktop-route-kinds";

describe("nav/project-route-requires-project", () => {
  it("keeps list routes page-owned instead of project-scoped", () => {
    expect(Desktop_PROJECT_ROUTE_KINDS).toHaveLength(0);
    expect(Desktop_ALL_ROUTE_KINDS.map((route) => route.id)).toContain("qa");
    expect(Desktop_ALL_ROUTE_KINDS.map((route) => route.id)).toContain("sessions");
  });

  it("keeps all-project routes usable without shell scope", () => {
    expect(Desktop_ALL_ROUTE_KINDS[0].scope).toBe("all");
    expect(Desktop_ALL_ROUTE_KINDS.map((route) => route.id)).toContain("plan");
  });
});
