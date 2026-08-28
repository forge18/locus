import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/disclosure-settings keeps cost off by default and hides tool rows on request", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("tool_call")]} />);
  expect(view.getByTestId("agent-cost-toggle").getAttribute("aria-pressed")).toBe("false");
  await fireEvent.click(view.getByTestId("agent-cost-toggle"));
  expect(view.getByTestId("agent-cost-toggle").textContent).toContain("$0.42");
  await fireEvent.click(view.getByTestId("agent-overflow-toggle"));
  await fireEvent.click(view.getByRole("menuitemradio", { name: "Hidden" }));
  expect(view.queryByTestId("agent-tool-card")).toBeNull();
});
