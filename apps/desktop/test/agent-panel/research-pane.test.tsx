import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/research-pane is session-scoped and labels provenance", async () => {
  const findings = [
    { id: "seed-1", title: "Seed", summary: "Inherited", source: "plan.md", provenance: "seed" as const },
    { id: "run-1", title: "Run", summary: "Observed", source: "run.md", provenance: "this_run" as const },
    { id: "close-1", title: "Close", summary: "Reviewed", source: "close.md", provenance: "session_close" as const },
  ];
  const view = mount(<AgentPane runId="run-1" live={false} session={session} findings={findings} />);
  await fireEvent.click(view.getByRole("button", { name: "Research" }));
  const pane = view.getByTestId("agent-research-pane");
  expect(pane.getAttribute("data-session-id")).toBe("run-1");
  expect(pane.querySelectorAll("[data-provenance]")).toHaveLength(3);
});
