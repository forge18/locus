import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

describe("Inbox evidence detail", () => {
  it("labels evidence, why, and waiting cost", () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    expect(
      getByTestId("inbox-explanations").querySelectorAll("p"),
    ).toHaveLength(2);
    expect(getByTestId("inbox-why")).toBeTruthy();
    expect(getByTestId("inbox-cost")).toBeTruthy();
  });
});
