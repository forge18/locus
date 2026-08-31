import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { configureInboxStub } from "./inbox-stub";

configureInboxStub();

describe("inbox/reflection-queue", () => {
  it("renders agent-sent deliveries in the same list, without a kind badge", async () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    await waitFor(() =>
      expect(getByTestId("inbox-card-d-0000")).toBeTruthy(),
    );
    // The live wire has no kind field: every card renders the same shape.
    expect(getByTestId("inbox-card-d-0000").getAttribute("data-kind")).toBeNull();
  });
});
