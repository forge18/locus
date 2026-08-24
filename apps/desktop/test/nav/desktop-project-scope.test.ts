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

  it("formats project routes with an explicit project scope and project segment", () => {
    expect(formatDesktopLocator("plan", "tapestry")).toBe(
      "locus://tapestry/view/plan",
    );
    expect(resolveDesktopLocator("locus://loom-db/view/plan")).toEqual({
      route: "plan",
      scope: { kind: "project", project: "loom-db" },
    });
  });

  it("rejects routes addressed with the wrong scope or an implicit v1 scope", () => {
    expect(() => formatDesktopLocator("inbox", "tapestry")).toThrow(/scope:/);
    expect(() => formatDesktopLocator("plan")).toThrow(/project:/);
    expect(() => resolveDesktopLocator("locus://all/view/plan")).toThrow(
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
