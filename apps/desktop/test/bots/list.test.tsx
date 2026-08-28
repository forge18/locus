import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";

describe("bots/list", () => {
  it("renders the project bot rail with live metadata and the contract footer", () => {
    const view = render(() => <BotsView projectId="tapestry" />);
    expect(view.getByTestId("bots-view").getAttribute("data-project")).toBe(
      "tapestry",
    );
    expect(view.getByTestId("bot-row-keeper").textContent).toContain("Keeper");
    expect(view.getByTestId("bot-row-keeper").textContent).toContain("pi");
    expect(
      view.getByTestId("bot-list").querySelectorAll("button"),
    ).toHaveLength(2);
    expect(view.getByText(/A bot is a named teammate/).textContent).toContain(
      "never touches the board",
    );
  });

  it("renders the contract empty state when the project has no bots", () => {
    const view = render(() => <BotsView bots={[]} />);
    expect(view.getByTestId("bots-empty-state").textContent).toBe(
      "No bots yet. Create one to have a standing agent you can message any time and hand recurring work to.",
    );
    expect(view.getByTestId("bot-home-pane").textContent).toBe("");
  });

  it("collapses to a dot strip without removing the bot selection", async () => {
    const view = render(() => <BotsView />);
    await fireEvent.click(
      view.getByRole("button", { name: "Collapse bot list" }),
    );
    expect(view.getByTestId("bots-view").getAttribute("data-collapsed")).toBe(
      "true",
    );
    expect(view.getByTestId("bot-row-keeper")).toBeTruthy();
    expect(
      view.getByTestId("bot-row-keeper").querySelector("strong"),
    ).toBeNull();
  });
});
