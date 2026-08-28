import { fireEvent, render } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { session } from "./support";

it("agent-panel/session-switch resets transient state only for a new run", async () => {
  const [runId, setRunId] = createSignal("run-1");
  const view = render(() => <AgentPane runId={runId()} live={false} session={session} blockers={runId() === "run-1" ? [{ id: "gate-1", kind: "gate", title: "Gate", detail: "Waiting" }] : undefined} />);
  await fireEvent.click(view.getByTestId("agent-cost-toggle"));
  await fireEvent.click(view.getByTestId("agent-blocker").querySelector("button")!);
  setRunId("run-2");
  await Promise.resolve();
  expect(view.getByTestId("agent-cost-toggle").getAttribute("aria-pressed")).toBe("true");
  expect(view.queryByTestId("agent-blocker")).toBeNull();
});
