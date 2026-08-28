import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/citation-to-research pins a turn citation into this session", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("assistant", 0, { raw: { citation: { id: "cite-1", label: "Source", source: "docs/source.md" } } })]} />);
  await fireEvent.click(view.getByTestId("agent-pin-citation"));
  await fireEvent.click(view.getByRole("button", { name: "Research" }));
  expect(view.getByTestId("agent-research-pane").getAttribute("data-session-id")).toBe("run-1");
  expect(view.getByText("Source")).toBeTruthy();
});
