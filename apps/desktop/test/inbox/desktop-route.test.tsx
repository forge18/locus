import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("Inbox desktop route", () => {
  it("identifies the global inbox fixture route", () => {
    const { getByTestId } = render(() => (
      <InboxView nav={createNavStore({ view: "inbox" })} />
    ));
    expect(getByTestId("inbox").getAttribute("data-desktop-route")).toBe(
      "inbox",
    );
  });
});
