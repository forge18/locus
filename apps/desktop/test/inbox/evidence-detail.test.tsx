import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

import { configureInboxStub } from "./inbox-stub";
configureInboxStub();

describe("Inbox evidence detail", () => {
  it("labels the cost of waiting on the live detail", async () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    await waitFor(() => expect(getByTestId("inbox-cost")).toBeTruthy());
    // The wire detail has one explanation (the cost of waiting); the fixture's
    // "why" prose was invented and is gone.
    expect(
      getByTestId("inbox-explanations").querySelectorAll("p"),
    ).toHaveLength(1);
  });
});
