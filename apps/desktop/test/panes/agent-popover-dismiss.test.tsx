import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import type { AgentPaneSession } from "../../src/panes/agent-panel-model";

const session: AgentPaneSession = {
  project: "p-tapestry",
  task: "task-42",
  workflow: "workflow-1",
  agent: "builder@4",
  model: "model-1",
  harness: "pi",
  effort: "high",
  name: "Thread the channel",
  context: { used: 12_000, total: 200_000 },
  cost: "$0.42",
  permissionPosture: "gated",
  status: "working",
};

describe("agent pane popover dismissal", () => {
  it("closes the overflow menu on Escape and refocuses the toggle", async () => {
    const view = render(() => (
      <AgentPane runId="run-1" live={false} session={session} />
    ));

    await fireEvent.click(view.getByTestId("agent-overflow-toggle"));
    expect(view.getByRole("menu")).toBeTruthy();

    await fireEvent.keyDown(document.body, { key: "Escape" });
    expect(view.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(
      view.getByTestId("agent-overflow-toggle"),
    );
  });

  it("closes the overflow menu on a press outside it", async () => {
    const view = render(() => (
      <AgentPane runId="run-1" live={false} session={session} />
    ));

    await fireEvent.click(view.getByTestId("agent-overflow-toggle"));
    expect(view.getByRole("menu")).toBeTruthy();

    await fireEvent.pointerDown(view.getByTestId("agent-stream"));
    expect(view.queryByRole("menu")).toBeNull();
  });

  it("keeps the overflow menu open for presses inside it and lets the toggle close it", async () => {
    const view = render(() => (
      <AgentPane runId="run-1" live={false} session={session} />
    ));

    await fireEvent.click(view.getByTestId("agent-overflow-toggle"));
    await fireEvent.pointerDown(view.getByRole("menu"));
    expect(view.queryByRole("menu")).not.toBeNull();

    await fireEvent.pointerDown(view.getByTestId("agent-overflow-toggle"));
    expect(view.queryByRole("menu")).not.toBeNull();
    await fireEvent.click(view.getByTestId("agent-overflow-toggle"));
    expect(view.queryByRole("menu")).toBeNull();
  });

  it("closes the context view on Escape and refocuses the context chip", async () => {
    const view = render(() => (
      <AgentPane runId="run-1" live={false} session={session} />
    ));

    await fireEvent.click(view.getByTestId("agent-context-toggle"));
    expect(view.getByTestId("agent-context-view")).toBeTruthy();

    await fireEvent.keyDown(document.body, { key: "Escape" });
    expect(view.queryByTestId("agent-context-view")).toBeNull();
    expect(document.activeElement).toBe(
      view.getByTestId("agent-context-toggle"),
    );
  });

  it("closes the context view on a press outside it", async () => {
    const view = render(() => (
      <AgentPane runId="run-1" live={false} session={session} />
    ));

    await fireEvent.click(view.getByTestId("agent-context-toggle"));
    expect(view.getByTestId("agent-context-view")).toBeTruthy();

    await fireEvent.pointerDown(view.getByTestId("agent-stream"));
    expect(view.queryByTestId("agent-context-view")).toBeNull();
  });
});
