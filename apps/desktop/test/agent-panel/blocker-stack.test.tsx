import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { elicitation, event, mount, plan, session } from "./support";

it("agent-panel/blocker-stack keeps gate and elicitation stacked and collapses the plan", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("permission_request")]} elicitation={elicitation} plan={plan} />);
  expect(view.getByTestId("agent-blocker").querySelectorAll(".agent-blocker")).toHaveLength(2);
  expect(view.getByTestId("agent-plan-dock").getAttribute("data-forced-collapsed")).toBe("true");
});
