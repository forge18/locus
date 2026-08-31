import { describe, expect, it } from "vitest";
import { DESKTOP_ROUTES } from "../../src/nav/desktop-screen-inventory";
import {
  Desktop_ALL_ROUTE_KINDS,
  Desktop_APP_ROUTE_KINDS,
  Desktop_PROJECT_ROUTE_KINDS,
  Desktop_ROUTE_KINDS,
} from "../../src/nav/desktop-route-kinds";

describe("nav/desktop-route-kinds", () => {
  it("registers every desktop route with its declared scope", () => {
    expect(Desktop_ROUTE_KINDS.map((route) => route.id)).toEqual(
      DESKTOP_ROUTES.map((route) => route.id),
    );
    expect(Desktop_ROUTE_KINDS.map((route) => route.scope)).toEqual(
      DESKTOP_ROUTES.map((route) => route.scope),
    );
    expect(new Set(Desktop_ROUTE_KINDS.map((route) => route.id)).size).toBe(30);
  });

  it("provides scope collections derived from the registered routes", () => {
    expect(Desktop_ALL_ROUTE_KINDS).toEqual(
      Desktop_ROUTE_KINDS.filter((route) => route.scope === "all"),
    );
    expect(Desktop_APP_ROUTE_KINDS).toEqual(
      Desktop_ROUTE_KINDS.filter((route) => route.scope === "app"),
    );
    expect(Desktop_PROJECT_ROUTE_KINDS).toEqual(
      Desktop_ROUTE_KINDS.filter((route) => route.scope === "project"),
    );
  });
});
