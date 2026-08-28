import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import BotsView from "../../src/screens/bots/BotsView";

describe("bots/panel", () => {
  it("composes the unmodified Agent Pane against the selected home session", async () => {
    const view = render(() => <BotsView projectId="tapestry" />);
    const pane = view.getByTestId("agent-pane");
    expect(pane.getAttribute("data-pty")).toBe("false");
    expect(pane.getAttribute("data-run-id")).toBe("bot-keeper-run");
    expect(view.getByTestId("agent-panel-header").textContent).toContain(
      "Keeper",
    );
    expect(view.getByTestId("agent-stream").textContent).toContain(
      "durable home conversation",
    );

    await fireEvent.click(view.getByTestId("bot-row-night-watch"));
    expect(view.getByTestId("agent-pane").getAttribute("data-run-id")).toBe(
      "bot-night-watch-run",
    );
  });
});
