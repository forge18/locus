import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/gate-mode changes the subsequent edit posture control", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} />);
  await fireEvent.click(view.getByRole("button", { name: "Auto" }));
  expect(view.getByRole("button", { name: "Auto" }).getAttribute("aria-pressed")).toBe("true");
});
