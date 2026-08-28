import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/tool-card exposes lifecycle states and disclosure", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("tool_call", 0, { tool: "read", raw: { status: "queued" } }), event("tool_result", 1, { tool: "write" }), event("tool_error", 2, { tool: "test" })]} />);
  const cards = view.getAllByTestId("agent-tool-card");
  expect(cards.map((card) => card.getAttribute("data-tool-status"))).toEqual(["queued", "completed", "failed"]);
  await fireEvent.click(cards[0].querySelector("button")!);
  expect(cards[0].querySelector("pre")).toBeNull();
});
