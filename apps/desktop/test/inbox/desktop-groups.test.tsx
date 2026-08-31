import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("Inbox desktop groups", () => {
  it("labels action-required and completed groups", () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    expect(
      getByTestId("inbox-tabs").querySelectorAll("[data-inbox-group]"),
    ).toHaveLength(2);
  });
});
