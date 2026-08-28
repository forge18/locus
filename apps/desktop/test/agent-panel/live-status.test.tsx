import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/live-status waits when a blocker is pending", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} blockers={[{ id: "gate-1", kind: "gate", title: "Review edit", detail: "The run is waiting." }]} />);
  expect(view.getByTestId("agent-live-status").textContent).toContain("waiting");
  expect(view.getByTestId("agent-live-status").textContent).toContain("needs you");
});
