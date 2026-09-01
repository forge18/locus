import { describe, expect, it } from "vitest";
import { resolveRouteScope } from "../../src/nav/route-scope";
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

  it("allows a global route without a selected project", () => {
    expect(resolveRouteScope(Desktop_ALL_ROUTE_KINDS[0], null)).toEqual({
      kind: "all",
    });
  });
});
