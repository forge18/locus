import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/docked-blocker docks a bounded blocker above the stream", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} blockers={[{ id: "gate-1", kind: "gate", title: "Approve edit", detail: "Review the proposed change." }]} />);
  expect(view.getByTestId("agent-blocker")).toBeTruthy();
  expect(view.getByTestId("agent-blocker").nextElementSibling?.className).toBe("agent-stream-shell");
});
