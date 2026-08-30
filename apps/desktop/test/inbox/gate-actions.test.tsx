import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

import { configureInboxStub } from "./inbox-stub";
configureInboxStub();

describe("Inbox gate actions", () => {
  it("labels approve and send-back actions", async () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    await waitFor(() => expect(getByTestId("inbox-approve")).toBeTruthy());
    expect(getByTestId("inbox-send-back")).toBeTruthy();
  });
});
