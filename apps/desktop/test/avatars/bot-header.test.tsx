import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";

describe("avatars/bot-header", () => {
  it("keeps the bot header outside the shared Agent Pane", async () => {
    const view = render(() => <BotsView />);
    const header = view.getByTestId("bot-view-header");

    expect(header.textContent).toContain("Keeper");
    expect(header.textContent).toContain("unknown");
    expect(view.getByTestId("bot-header-avatar").getAttribute("alt")).toBe(
      "Keeper avatar",
    );
    expect(view.getByTestId("agent-pane")).toBeTruthy();

    await fireEvent.click(view.getByTestId("bot-row-night-watch"));
    expect(view.getByTestId("bot-view-header").textContent).toContain(
      "Night Watch",
    );
    expect(view.getByTestId("bot-header-avatar").getAttribute("alt")).toBe(
      "Night Watch avatar",
    );
  });
});
