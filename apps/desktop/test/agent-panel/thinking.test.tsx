import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/thinking defaults to a summary and allows full or hidden", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("thinking", 0, { text: "First sentence. More detail follows." })]} />);
  expect(view.getByTestId("agent-thinking-block").textContent).not.toContain("More detail follows");
  await fireEvent.click(view.getByRole("button", { name: "Full" }));
  expect(view.getByTestId("agent-thinking-block").textContent).toContain("More detail follows");
  await fireEvent.click(view.getByRole("button", { name: "Hidden" }));
  expect(view.getByTestId("agent-thinking-block").textContent).not.toContain("First sentence");
});
