import { expect, it } from "vitest";
import { typedElicitationValues } from "../../src/panes/agent-pane-utils";
import type { AgentPaneElicitation } from "../../src/panes/agent-panel-model";

it("agent-panel/typed-values emits JSON-compatible primitive values", () => {
  const request: AgentPaneElicitation = { id: "typed", title: "Typed", detail: "", fields: [{ id: "count", label: "Count", type: "integer" }, { id: "ok", label: "OK", type: "boolean" }, { id: "name", label: "Name", type: "text" }] };
  expect(typedElicitationValues(request, { count: "2", ok: "true", name: "Locus" })).toEqual({ count: 2, ok: true, name: "Locus" });
});
