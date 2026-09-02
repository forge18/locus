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
      "Waiting for ACP events from this run.",
    );
    expect(view.getByTestId("agent-cost-toggle").textContent).toContain(
      "$0.42",
    );
    expect(view.getByTestId("pane-manager")).toBeTruthy();
    await fireEvent.click(view.getByTestId("pane-split-keeper"));
    expect(view.getAllByTestId("agent-pane")).toHaveLength(2);
    await fireEvent.click(view.getByTestId("pane-minimize-keeper"));
    expect(view.getByTestId("pane-promote-keeper")).toBeTruthy();
    await fireEvent.click(view.getByTestId("pane-promote-keeper"));
    expect(view.getAllByTestId("agent-pane")).toHaveLength(2);

    await fireEvent.click(view.getByTestId("bot-row-night-watch"));
    expect(view.getByTestId("agent-pane").getAttribute("data-run-id")).toBe(
      "bot-night-watch-session",
    );
  });
});
