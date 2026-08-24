import { describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav/store";

describe("nav/window-history", () => {
  it("keeps independent back and forward history within one window store", () => {
    const nav = createNavStore();
    nav.go("plan");
    nav.go("sessions");

    nav.back();
    expect(nav.view()).toBe("plan");
    expect(nav.canForward()).toBe(true);
    nav.forward();
    expect(nav.view()).toBe("sessions");
  });
});
