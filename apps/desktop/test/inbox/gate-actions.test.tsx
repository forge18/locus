import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

describe("Inbox gate actions", () => {
  it("labels approve and send-back actions", () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    expect(getByTestId("inbox-approve")).toBeTruthy();
    expect(getByTestId("inbox-send-back")).toBeTruthy();
  });
});
