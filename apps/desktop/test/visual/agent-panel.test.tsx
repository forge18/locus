import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, plan, session } from "../agent-panel/support";

it("visual/agent-panel keeps the handoff hierarchy and token-backed geometry", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("assistant", 0, { text: "output" })]} plan={plan} />);
  expect(view.getByTestId("agent-panel-header")).toBeTruthy();
  expect(view.getByTestId("agent-stream")).toBeTruthy();
  expect(view.getByTestId("agent-composer")).toBeTruthy();
  const css = readFileSync(resolve(process.cwd(), "src/panes/agent-pane.css"), "utf8");
  expect(css).toContain("min-height: 44px");
  expect(css).toContain("flex: 0 0 380px");
  expect(css).toContain("var(--t-micro)");
});
