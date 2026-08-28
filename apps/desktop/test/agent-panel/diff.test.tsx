import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/diff renders added, removed, context, and line gutters", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("permission_request", 0, { raw: { diff: { path: "src/file.ts", before: "old\nsame", after: "new\nsame" } } })]} />);
  expect(view.getAllByTestId("agent-diff-row").map((row) => row.getAttribute("data-diff-kind"))).toEqual(["removed", "added", "context"]);
  expect(view.getByTestId("agent-inline-diff").querySelectorAll(".agent-diff-line-number")).toHaveLength(6);
});
