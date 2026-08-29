import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";

describe("avatars/bot-list", () => {
  it("shows one derived avatar and a live-state marker per bot row", () => {
    const view = render(() => <BotsView />);

    expect(view.getByTestId("bot-avatar-keeper").tagName).toBe("IMG");
    expect(view.getByTestId("bot-avatar-keeper").getAttribute("alt")).toBe(
      "Keeper avatar",
    );
    expect(
      view.getByTestId("bot-avatar-keeper").getAttribute("data-avatar-style"),
    ).toBe("bottts");
    expect(
      view
        .getByTestId("bot-row-keeper")
        .querySelector(".bot-live-dot")
        ?.getAttribute("data-live-state"),
    ).toBe("working");
    expect(
      view
        .getByTestId("bot-row-night-watch")
        .querySelector(".bot-live-dot")
        ?.getAttribute("data-live-state"),
    ).toBe("idle");
  });

  it("keeps the avatar and live badge visible in the collapsed strip", async () => {
    const view = render(() => <BotsView />);
    await fireEvent.click(
      view.getByRole("button", { name: "Collapse bot list" }),
    );

    expect(view.getByTestId("bot-avatar-keeper")).toBeTruthy();
    expect(
      view.getByTestId("bot-row-keeper").querySelector(".bot-live-dot"),
    ).toBeTruthy();
    expect(
      view.getByTestId("bot-row-keeper").querySelector("strong"),
    ).toBeNull();
  });
});
