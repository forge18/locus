import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { useInboxItems } from "../../src/data/inbox";

const mount = () => {
  const nav = createNavStore();
  const r = render(() => (
    <Shell nav={nav}>
      <InboxView nav={nav} />
    </Shell>
  ));
  return { nav, ...r };
};

describe("inbox/resolves-in-place", () => {
  it("removes the item from the list", () => {
    const { getByTestId, queryByTestId } = mount();
    const [first] = useInboxItems();
    expect(queryByTestId(`inbox-card-${first.id}`)).not.toBe(null);
    getByTestId("inbox-approve").click();
    expect(queryByTestId(`inbox-card-${first.id}`)).toBe(null);
  });

  it("leaves the view exactly where it was", () => {
    const { nav, getByTestId } = mount();
    const before = nav.locator();
    getByTestId("inbox-approve").click();
    expect(nav.view()).toBe("inbox");
    expect(nav.locator()).toBe(before);
  });

  it("leaves the rail where it was", () => {
    const { getByTestId } = mount();
    getByTestId("inbox-approve").click();
    expect(getByTestId("project-rail")).toBeTruthy();
    expect(getByTestId("title-category").textContent).toBe("Inbox");
  });

  it("does not push onto the history — resolving is not navigating", () => {
    const { nav, getByTestId } = mount();
    const before = nav.history().length;
    getByTestId("inbox-approve").click();
    expect(nav.history().length).toBe(before);
  });

  it("moves the detail pane on to the next item that needs a person", () => {
    const { getByTestId } = mount();
    const [first, second] = useInboxItems();
    expect(getByTestId("inbox-detail-title").textContent).toBe(first.title);
    getByTestId("inbox-approve").click();
    expect(getByTestId("inbox-detail-title").textContent).toBe(second.title);
  });

  it("drops the badge count as items resolve", () => {
    const { getByTestId } = mount();
    // The inbox itself presents the unresolved count in its visible heading.
    const initialCount = useInboxItems().length;
    expect(getByTestId("needs-you-note").textContent).toContain(
      `${initialCount} items`,
    );
    getByTestId("inbox-approve").click();
    expect(getByTestId("needs-you-note").textContent).toContain(
      `${initialCount - 1} items`,
    );
  });

  it("resolves a send-back too — the decision is made either way", () => {
    const { getByTestId, queryByTestId } = mount();
    const [first] = useInboxItems();
    getByTestId("inbox-send-back").click();
    expect(queryByTestId(`inbox-card-${first.id}`)).toBe(null);
  });
});
