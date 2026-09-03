import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { configureInboxStub, RESOLVED_TODAY } from "./inbox-stub";
import { read, rules } from "../css";

const mount = () => render(() => <InboxView nav={createNavStore()} />);

configureInboxStub();

describe("inbox/resolved", () => {
  it("is headed RESOLVED TODAY", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(getByTestId("resolved-title").textContent).toBe("Resolved today"),
    );
  });

  it("lists one row per resolved item", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(
        getByTestId("inbox-resolved").querySelectorAll(".inbox-resolved-row")
          .length,
      ).toBe(RESOLVED_TODAY.length),
    );
  });

  it("uses muted semantic color — done is context, not work", () => {
    const rule = rules(read("screens/screens.css")).find(
      (r) => r.selector === ".inbox-resolved-row",
    )!;
    expect(rule.body).toContain("color: var(--text-muted)");
    expect(rule.body).not.toMatch(/opacity:\s*\.6/);
  });

  it("shows the subject and the age on each row", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(
        getByTestId(`resolved-${RESOLVED_TODAY[0].id}`).textContent,
      ).toContain(RESOLVED_TODAY[0].subject),
    );
    expect(
      getByTestId(`resolved-${RESOLVED_TODAY[0].id}`).textContent,
    ).toContain("1h");
  });

  it("sits below the live items, not among them", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(
        getByTestId("inbox-resolved").querySelectorAll(".inbox-resolved-row")
          .length,
      ).toBeGreaterThan(0),
    );
    const list = getByTestId("inbox-list");
    const cards = list.querySelectorAll(".inbox-card");
    const resolved = getByTestId("inbox-resolved");
    expect(
      cards[cards.length - 1].compareDocumentPosition(resolved) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });
});
