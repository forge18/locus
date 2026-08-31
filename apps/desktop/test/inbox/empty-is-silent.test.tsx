import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";
import { read } from "../css";

configureInboxStub();

/** Resolve everything, which is the only way to reach empty from the seeded rows. */
const emptied = async () => {
  const r = render(() => <InboxView nav={createNavStore()} />);
  await waitFor(() =>
    expect(r.getByTestId("needs-you-note").textContent).toContain(
      `${PENDING.length} items`,
    ),
  );
  for (let i = 0; i < PENDING.length; i++) {
    r.getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(r.getByTestId("needs-you-note").textContent).toContain(
        `${PENDING.length - 1 - i} item`,
      ),
    );
  }
  return r;
};

describe("inbox/empty-is-silent", () => {
  it('says "Nothing needs you"', async () => {
    const { container } = await emptied();
    expect(container.textContent).toContain("Nothing needs you");
  });

  it("shows no cards at all", async () => {
    const { container } = await emptied();
    expect(container.querySelectorAll(".inbox-card").length).toBe(0);
  });

  it("shows no spinner — silence is the default here, not loading", async () => {
    const { container } = await emptied();
    expect(container.querySelector(".pulse")).toBe(null);
    expect(container.querySelector(".skeleton-rows")).toBe(null);
    expect(container.textContent?.toLowerCase()).not.toContain("loading");
  });

  it('states a reason rather than "No items"', async () => {
    const { container } = await emptied();
    expect(container.textContent).not.toContain("No items");
    expect(container.querySelector('[data-testid="empty-pane"]')).not.toBe(
      null,
    );
  });

  it("counts zero in the header without breaking the sentence", async () => {
    const { getByTestId } = await emptied();
    expect(getByTestId("needs-you-note").textContent).toBe(
      "0 items · silence is the default",
    );
  });

  it("never puts a notification here — the screen has no such path", () => {
    // An item that only reports something happened is not inbox work, and the
    // component takes no shape that could carry one. The notify import is the
    // resolve-failure toast, not a notification item.
    expect(read("screens/inbox/InboxView.tsx")).not.toMatch(
      /notification-item|addNotification/,
    );
  });
});
