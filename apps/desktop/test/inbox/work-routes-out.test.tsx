import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore, parse } from "../../src/nav";
import { useInboxItems } from "../../src/data/inbox";

const mount = () => {
  const nav = createNavStore();
  const r = render(() => <InboxView nav={nav} />);
  return { nav, ...r };
};

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("inbox/work-routes-out", () => {
  it("lands a plan gate in Plan", () => {
    const { nav, getByTestId } = mount();
    getByTestId("inbox-detail-locator").click();
    expect(nav.view()).toBe("plan");
    expect(nav.params().project).toBe("tapestry");
  });

  it("lands an ask in Interact, where the question is answered", () => {
    const { nav, getByTestId } = mount();
    const [, ask] = useInboxItems();
    getByTestId(`inbox-card-${ask.id}`).click();
    getByTestId("inbox-detail-locator").click();
    expect(nav.view()).toBe("interact");
    expect(nav.params().project).toBe("loom-db");
  });

  it("lands a guardrail in Runs, because a finished run is examined there", () => {
    const { nav, getByTestId } = mount();
    const [, , guardrail] = useInboxItems();
    getByTestId(`inbox-card-${guardrail.id}`).click();
    getByTestId("inbox-detail-locator").click();
    expect(parse(guardrail.opensAt).project).toBe("all");
    expect(nav.view()).toBe("runs");
    expect(nav.params()).toEqual({});
  });

  it("lands the three items in three different categories", () => {
    const seen = new Set<string>();
    for (const item of useInboxItems()) {
      const { nav, getByTestId, unmount } = mount();
      getByTestId(`inbox-card-${item.id}`).click();
      getByTestId("inbox-detail-locator").click();
      seen.add(nav.view());
      unmount();
    }
    expect([...seen].sort()).toEqual(["interact", "plan", "runs"]);
  });

  it("carries the project across, because it is in the locator", () => {
    const { nav, getByTestId } = mount();
    const [, ask] = useInboxItems();
    getByTestId(`inbox-card-${ask.id}`).click();
    getByTestId("inbox-detail-locator").click();
    expect(nav.locator()).toContain("loom-db");
  });

  it("pushes onto the history, because this one really is navigation", () => {
    const { nav, getByTestId } = mount();
    const before = nav.history().length;
    getByTestId("inbox-detail-locator").click();
    expect(nav.history().length).toBe(before + 1);
  });

  it("every item names a locator that parses", () => {
    for (const item of useInboxItems()) {
      expect(() => parse(item.opensAt), item.id).not.toThrow();
    }
  });
});
