import { describe, expect, it } from "vitest";
import { VIEWS, drilldownParent } from "../../src/nav";

describe("nav/drilldown-parent", () => {
  it("sends the canvas back to the Workflows list it was opened from", () => {
    expect(drilldownParent("canvas")).toBe("workflows");
  });

  it("gives every other view no parent", () => {
    for (const view of VIEWS) {
      if (view === "canvas") continue;
      expect(drilldownParent(view), view).toBeNull();
    }
  });

  it("never chains — a parent is not itself a drill-down", () => {
    for (const view of VIEWS) {
      const parent = drilldownParent(view);
      if (parent) expect(drilldownParent(parent), view).toBeNull();
    }
  });

  it("parents a drill-down with a registered view", () => {
    for (const parent of VIEWS.flatMap((view) => drilldownParent(view) ?? [])) {
      expect(VIEWS).toContain(parent);
    }
  });

  it("keeps agent definitions a landing view, not a drill-down", () => {
    // Acceptance 4's back-link clause predates the M0.7 shell revision, which
    // made `agents` the Workshop landing view; there is no Extensions view to
    // go back to. The clause that survives — no Agents tab in the Workshop
    // bar — is pinned by nav/tab-sets.
    expect(drilldownParent("agents")).toBeNull();
  });
});
