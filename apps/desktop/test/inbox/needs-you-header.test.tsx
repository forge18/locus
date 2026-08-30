import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";
import { read, rules } from "../css";

const mount = () => render(() => <InboxView nav={createNavStore()} />);

configureInboxStub();

describe("inbox/needs-you-header", () => {
  it("reads NEEDS YOU", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(getByTestId("needs-you-title").textContent).toBe("Needs you"),
    );
    const rule = rules(read("screens/screens.css")).find(
      (r) => r.selector === ".inbox-section-title",
    )!;
    expect(rule.body).toContain("text-transform: uppercase");
  });

  it("sets the label in accent", () => {
    const rule = rules(read("screens/screens.css")).find(
      (r) => r.selector === ".inbox-section-title",
    )!;
    expect(rule.body).toContain("color: var(--action-attention)");
  });

  it("counts the items that are actually there", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(getByTestId("needs-you-note").textContent).toContain(
        `${PENDING.length} items`,
      ),
    );
  });

  it("carries the note that silence is the default", async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(getByTestId("needs-you-note").textContent).toContain(
        "silence is the default",
      ),
    );
  });

  it('says "item" rather than "items" for one', async () => {
    const { getByTestId } = mount();
    await waitFor(() =>
      expect(getByTestId("needs-you-note").textContent).toContain(
        `${PENDING.length} items`,
      ),
    );
    // Resolve all but one — each resolve is async.
    for (let index = 1; index < PENDING.length; index += 1) {
      getByTestId("inbox-approve").click();
      await waitFor(() =>
        expect(getByTestId("needs-you-note").textContent).toContain(
          `${PENDING.length - 1 - index + 1} item`,
        ),
      );
    }
  });
});
