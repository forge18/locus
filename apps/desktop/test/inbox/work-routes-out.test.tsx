import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";

const mount = () => {
  const nav = createNavStore();
  const r = render(() => <InboxView nav={nav} />);
  return { nav, ...r };
};


configureInboxStub();

describe("inbox/work-routes-out", () => {
  it("resolves in place — nothing about where you are changes", async () => {
    const { nav, getByTestId } = mount();
    const first = await waitFor(() =>
      getByTestId(`inbox-card-${PENDING[0].id}`),
    );
    first.click();
    getByTestId("inbox-approve").click();
    // The resolved delivery leaves the to-do panel; it stays in the completed
    // panel as the audit trail.
    await waitFor(() =>
      expect(getByTestId("inbox-todo-count").textContent).toBe("2"),
    );
    expect(getByTestId("inbox-items").textContent).not.toContain(
      PENDING[0].subject,
    );
    // The route never moved: resolving is not navigating.
    expect(nav.view()).toBe("inbox");
  });

  it("every pending delivery names a thread the decision can be audited on", () => {
    for (const item of PENDING) {
      expect(item.threadId.length).toBeGreaterThan(0);
    }
  });
});
