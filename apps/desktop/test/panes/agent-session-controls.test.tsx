import { fireEvent, render } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import type { AgentPaneSession } from "../../src/panes/agent-panel-model";

const runningSession: AgentPaneSession = {
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

it("interrupts a running session without submitting the composer draft", async () => {
  const onSend = vi.fn();
  const onQueue = vi.fn();
  const onStop = vi.fn();
  const view = render(() => (
    <AgentPane
      runId="run-1"
      live={false}
      session={runningSession}
      onSend={onSend}
      onQueue={onQueue}
      onStop={onStop}
    />
  ));
  const input = view.getByLabelText("Message agent") as HTMLTextAreaElement;
  await fireEvent.input(input, { target: { value: "keep this draft" } });
  const stop = view.getByRole("button", { name: "Stop" });

  expect(stop.getAttribute("type")).toBe("button");
  await fireEvent.click(stop);

  expect(onStop).toHaveBeenCalledTimes(1);
  expect(onSend).not.toHaveBeenCalled();
  expect(onQueue).not.toHaveBeenCalled();
  expect(input.value).toBe("keep this draft");
});
