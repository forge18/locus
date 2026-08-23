import { describe, expect, it } from "vitest";
import {
  format,
  formatDesktopLocator,
  resolve,
  resolveDesktopLocator,
} from "../../src/nav";

describe("nav/desktop-project-scope", () => {
  it("formats global routes with an explicit global scope", () => {
    expect(formatDesktopLocator("inbox")).toBe("locus://global/inbox");
    expect(resolveDesktopLocator("locus://global/dispatch-runs")).toEqual({
      route: "dispatch-runs",
      scope: { kind: "global" },
    });
  });

  it("formats project routes with an explicit project scope and project segment", () => {
    expect(formatDesktopLocator("plan-conversation", "tapestry")).toBe(
      "locus://project/tapestry/plan-conversation",
    );
    expect(
      resolveDesktopLocator("locus://project/loom-db/review-telemetry"),
    ).toEqual({
      route: "review-telemetry",
      scope: { kind: "project", project: "loom-db" },
    });
  });

  it("rejects routes addressed with the wrong scope or an implicit v1 scope", () => {
    expect(() => formatDesktopLocator("inbox", "tapestry")).toThrow(/scope:/);
    expect(() => formatDesktopLocator("plan-conversation")).toThrow(/project:/);
    expect(() => resolveDesktopLocator("locus://global/plan-conversation")).toThrow(
      /scope:/,
    );
    expect(() => resolveDesktopLocator("locus://project/tapestry/inbox")).toThrow(
      /scope:/,
    );
    expect(() => resolveDesktopLocator("locus://tapestry/inbox")).toThrow(/scope:/);
  });

  it("preserves the v1 fixture resolver during the migration", () => {
    expect(format("inbox", { project: "tapestry" })).toBe(
      "locus://tapestry/inbox",
    );
    expect(resolve("locus://tapestry/inbox")).toEqual({
      view: "inbox",
      params: { project: "tapestry" },
    });
  });
});
