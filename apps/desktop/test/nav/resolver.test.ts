import { describe, expect, it } from "vitest";
import { createNavStore, resolve } from "../../src/nav";

describe("nav/resolver", () => {
  it("turns a locator into a view and its params", () => {
    expect(resolve("locus://tapestry/session/8f21")).toEqual({
      view: "sessions",
      params: { project: "tapestry", sessionId: "8f21" },
    });
  });

  it("opens one object in one viewer, however it was reached", () => {
    // Three entry points, one target: the inbox item, the board card, the deep link.
    const fromInbox = resolve("locus://weaver/artifact/a-1");
    const fromBoard = resolve("locus://weaver/artifact/a-1");
    const fromLink = resolve("locus://weaver/artifact/a-1");
    expect(fromInbox).toEqual(fromBoard);
    expect(fromBoard).toEqual(fromLink);
    expect(fromInbox.view).toBe("artifact");
  });

  it("crosses categories cleanly — the same session is one object", () => {
    const session = resolve("locus://tapestry/session/8f21");
    expect(session.view).toBe("sessions");
    expect(session.params.sessionId).toBe("8f21");
  });

  it("is the entry point the store navigates through", () => {
    const nav = createNavStore();
    const target = nav.open("locus://loom-db/task/t-004");
    expect(target).toEqual({
      view: "sessions",
      params: { project: "loom-db", taskId: "t-004" },
    });
    expect(nav.view()).toBe("sessions");
    expect(nav.params().taskId).toBe("t-004");
  });

  it("throws rather than navigating somewhere wrong", () => {
    const nav = createNavStore();
    expect(() => nav.open("locus://tapestry/widget/x")).toThrow(/kind:/);
    expect(nav.view()).toBe("inbox");
  });

  it("resolves the project out of the path, so a filter is a query not a screen", () => {
    expect(resolve("locus://weaver/view/sessions").params.project).toBe(
      "weaver",
    );
    expect(resolve("locus://texere/view/sessions").params.project).toBe(
      "texere",
    );
  });
});
