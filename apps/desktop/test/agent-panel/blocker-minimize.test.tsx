import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/blocker-minimize restores the blocker without changing its id", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} blockers={[{ id: "gate-1", kind: "gate", title: "Approve", detail: "Waiting" }]} />);
  const blocker = view.getByTestId("agent-blocker").querySelector("[data-blocker-id]") as HTMLElement;
  await fireEvent.click(blocker.querySelector("button")!);
  expect(blocker.getAttribute("data-blocker-minimized")).toBe("true");
  await fireEvent.click(blocker.querySelector("button")!);
  expect(blocker.getAttribute("data-blocker-minimized")).toBe("false");
  expect(blocker.getAttribute("data-blocker-id")).toBe("gate-1");
});
