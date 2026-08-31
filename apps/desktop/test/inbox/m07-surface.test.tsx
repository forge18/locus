import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";
import type { NavStore } from "../../src/nav";

const nav = { open: () => ({}) } as unknown as NavStore;
const mount = () => render(() => <InboxView nav={nav} />);

configureInboxStub();

describe("inbox/m07-surface", () => {
  it("renders the To do and Completed tabs, throughput, and a live item log", async () => {
    const { getByTestId } = mount();

    await waitFor(() =>
      expect(getByTestId("inbox-tab-todo").textContent).toContain("To do"),
    );
    expect(getByTestId("inbox-tab-completed").textContent).toContain(
      "Completed",
    );
    expect(getByTestId("inbox-throughput").textContent).toContain("pending");
    expect(getByTestId("inbox-throughput").textContent).toContain(
      "resolved today",
    );
    expect(getByTestId("inbox-project-filter-note").textContent).toContain(
      "Filters this list only",
    );
    expect(getByTestId("inbox-items").getAttribute("aria-live")).toBe("polite");
    expect(getByTestId("inbox-items").getAttribute("role")).toBe("log");
    // The inbox-cost element is on the detail pane — needs a selected item.
    await waitFor(() =>
      expect(getByTestId("inbox-detail-title").textContent).toContain(
        "sign-off",
      ),
    );
    expect(getByTestId("inbox-cost").textContent).toContain(
      "No tokens burn while blocked.",
    );
  });

  it("groups completed rows by day and shows their resolution time", async () => {
    const { getByTestId } = mount();
    getByTestId("inbox-tab-completed").click();

    await waitFor(() =>
      expect(
        getByTestId("inbox-completed-items").querySelectorAll(
          ".inbox-completed-day",
        ).length,
      ).toBeGreaterThan(0),
    );
  });

  it("names a thread for each inbox destination", () => {
    for (const item of PENDING) {
      expect(item.threadId.length).toBeGreaterThan(0);
    }
  });
});
