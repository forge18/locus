import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createNavStore } from "../../src/nav";
import { NAV_HISTORY_STORAGE_KEY, NAV_STORAGE_KEY } from "../../src/nav/store";

const resetNavigationStorage = () => {
  localStorage.removeItem(NAV_STORAGE_KEY);
  localStorage.removeItem(NAV_HISTORY_STORAGE_KEY);
  history.replaceState(null, "");
};

beforeEach(resetNavigationStorage);
afterEach(resetNavigationStorage);

describe("nav/navigation-persistence", () => {
  it("restores the locator and back/forward stack after a reload", () => {
    const firstWindow = createNavStore();
    firstWindow.go("sessions");
    firstWindow.go("runs", {
      project: "tapestry",
      sessionId: "8f21",
      runId: "3c04",
    });
    firstWindow.back();
    window.dispatchEvent(new Event("pagehide"));

    const restartedWindow = createNavStore();

    expect(restartedWindow.locator()).toBe("locus://all/view/sessions");
    expect(restartedWindow.history()).toEqual([
      "locus://all/view/inbox",
      "locus://all/view/sessions",
      "locus://tapestry/session/8f21/run/3c04",
    ]);
    expect(restartedWindow.canBack()).toBe(true);
    expect(restartedWindow.canForward()).toBe(true);

    restartedWindow.forward();
    expect(restartedWindow.params()).toEqual({
      project: "tapestry",
      sessionId: "8f21",
      runId: "3c04",
    });
  });

  it("follows browser popstate entries without losing forward history", () => {
    const nav = createNavStore();
    nav.go("plan");
    const planState = history.state;
    nav.go("sessions");
    const sessionsState = history.state;

    window.dispatchEvent(new PopStateEvent("popstate", { state: planState }));
    expect(nav.view()).toBe("plan");
    expect(nav.canForward()).toBe(true);

    window.dispatchEvent(
      new PopStateEvent("popstate", { state: sessionsState }),
    );
    expect(nav.view()).toBe("sessions");
    expect(nav.params()).toEqual({});
    expect(nav.canForward()).toBe(false);
  });
});
