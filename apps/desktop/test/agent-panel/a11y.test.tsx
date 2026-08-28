import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/a11y exposes labels, state, and keyboard discovery", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("permission_request")]} />);
  expect(view.getByLabelText("Message agent").getAttribute("aria-controls")).toBe("agent-composer-suggestions");
  expect(view.getByTestId("agent-live-status").getAttribute("type")).toBe("button");
  await fireEvent.keyDown(view.getByLabelText("Message agent"), { key: "ArrowDown" });
  expect(view.getByRole("button", { name: "Manual" }).getAttribute("aria-pressed")).toBe("true");
});
