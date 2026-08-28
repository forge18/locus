import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, plan, session } from "./support";

it("agent-panel/plan-dock renders one collapsible active plan", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} plan={plan} />);
  await fireEvent.click(view.getByRole("button", { name: /Build and verify/ }));
  expect(view.getByTestId("agent-plan-dock").textContent).toContain("Wire the stream");
  expect(view.getByTestId("agent-plan-dock").getAttribute("data-plan-id")).toBe("plan-1");
});
