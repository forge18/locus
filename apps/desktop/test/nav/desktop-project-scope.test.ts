import { describe, expect, it } from "vitest";
import {
  format,
  formatDesktopLocator,
  resolve,
  resolveDesktopLocator,
} from "../../src/nav";

describe("nav/desktop-project-scope", () => {
  it("formats all-project routes with an explicit all scope", () => {
    expect(formatDesktopLocator("inbox")).toBe("locus://all/view/inbox");
    expect(resolveDesktopLocator("locus://all/view/runs")).toEqual({
      route: "runs",
      scope: { kind: "all" },
    });
  });

  it("formats page-owned routes with the all-project scope", () => {
    expect(formatDesktopLocator("plan", "tapestry")).toThrow(/scope:/);
    expect(formatDesktopLocator("plan")).toBe("locus://all/view/plan");
    expect(resolveDesktopLocator("locus://all/view/plan")).toEqual({
      route: "plan",
      scope: { kind: "all" },
    });
    expect(resolveDesktopLocator("locus://all/view/qa")).toEqual({
      route: "qa",
      scope: { kind: "all" },
    });
  });

  it("rejects routes addressed with the wrong scope or an implicit v1 scope", () => {
    expect(() => formatDesktopLocator("inbox", "tapestry")).toThrow(/scope:/);
    expect(() => resolveDesktopLocator("locus://app/view/plan")).toThrow(
      /scope:/,
    );
    expect(() => resolveDesktopLocator("locus://tapestry/view/inbox")).toThrow(
      /scope:/,
    );
    expect(() => resolveDesktopLocator("locus://tapestry/inbox")).toThrow(
      /locator:/,
    );
  });

  it("uses the same canonical view grammar through the fixture resolver", () => {
    expect(format("inbox", { project: "tapestry" })).toBe(
      "locus://all/view/inbox",
    );
    expect(resolve("locus://all/view/inbox")).toEqual({
      view: "inbox",
      params: {},
    });
  });
});
