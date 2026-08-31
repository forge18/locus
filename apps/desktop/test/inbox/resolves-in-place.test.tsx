import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";

const mount = () => {
  const nav = createNavStore();
  const r = render(() => (
    <Shell nav={nav}>
      <InboxView nav={nav} />
    </Shell>
  ));
  return { nav, ...r };
};

const typeComment = (box: Element, text: string) => {
  const textarea = box as HTMLTextAreaElement;
  textarea.value = text;
  textarea.dispatchEvent(new Event("input", { bubbles: true }));
};

configureInboxStub();

describe("inbox/resolves-in-place", () => {
  it("removes the item from the list", async () => {
    const { getByTestId, queryByTestId } = mount();
    const [first] = PENDING;
    await waitFor(() =>
      expect(queryByTestId(`inbox-card-${first.id}`)).not.toBe(null),
    );
    getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(queryByTestId(`inbox-card-${first.id}`)).toBe(null),
    );
  });

  it("leaves the view exactly where it was", async () => {
    const { nav, getByTestId } = mount();
    await waitFor(() => getByTestId("inbox-approve"));
    const before = nav.locator();
    getByTestId("inbox-approve").click();
    await waitFor(() => expect(nav.view()).toBe("inbox"));
    expect(nav.locator()).toBe(before);
  });

  it("leaves the rail where it was", async () => {
    const { getByTestId } = mount();
    await waitFor(() => getByTestId("inbox-approve"));
    getByTestId("inbox-approve").click();
    expect(getByTestId("project-rail")).toBeTruthy();
    expect(getByTestId("title-category").textContent).toBe("Inbox");
  });

  it("does not push onto the history — resolving is not navigating", async () => {
    const { nav, getByTestId } = mount();
    await waitFor(() => getByTestId("inbox-approve"));
    const before = nav.history().length;
    getByTestId("inbox-approve").click();
    expect(nav.history().length).toBe(before);
  });

  it("moves the detail pane on to the next item that needs a person", async () => {
    const { getByTestId } = mount();
    const [, second] = PENDING;
    await waitFor(() =>
      expect(getByTestId("inbox-detail-title").textContent).toContain(
        PENDING[0].subject,
      ),
    );
    getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(getByTestId("inbox-detail-title").textContent).toContain(
        second.subject,
      ),
    );
  });

  it("drops the badge count as items resolve", async () => {
    const { getByTestId } = mount();
    const initialCount = PENDING.length;
    await waitFor(() =>
      expect(getByTestId("needs-you-note").textContent).toContain(
        `${initialCount} items`,
      ),
    );
    getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(getByTestId("needs-you-note").textContent).toContain(
        `${initialCount - 1} items`,
      ),
    );
  });

  it("resolves a send-back too — the decision is made either way", async () => {
    const { getByTestId, queryByTestId } = mount();
    const [first] = PENDING;
    await waitFor(() => getByTestId("inbox-comment"));
    typeComment(getByTestId("inbox-comment"), "Rework the sink boundary.");
    getByTestId("inbox-send-back").click();
    await waitFor(() =>
      expect(queryByTestId(`inbox-card-${first.id}`)).toBe(null),
    );
  });

  it("does not resolve a send-back without its comment — nothing was decided", async () => {
    const { getByTestId, queryByTestId } = mount();
    const [first] = PENDING;
    await waitFor(() => getByTestId("inbox-send-back"));
    getByTestId("inbox-send-back").click();
    expect(queryByTestId(`inbox-card-${first.id}`)).not.toBe(null);
    expect(getByTestId("inbox-send-back-error")).toBeTruthy();
  });
});
