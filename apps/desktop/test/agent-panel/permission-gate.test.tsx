import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/permission-gate resolves gated edits with distinct actions", async () => {
  const onApprove = vi.fn();
  const onDecline = vi.fn();
  const onRemaining = vi.fn();
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("permission_request", 0, { raw: { diff: { path: "src/file.ts", before: "old", after: "new" } } })]} onApprovePermission={onApprove} onDeclinePermission={onDecline} onApproveRemainingTurn={onRemaining} />);
  await fireEvent.click(view.getByRole("button", { name: "Approve remaining this turn" }));
  expect(onRemaining).toHaveBeenCalledTimes(1);
  expect(view.getByTestId("agent-permission-card").getAttribute("data-resolved")).toBe("true");
  expect(view.getByTestId("agent-live-status").textContent).toContain("working");
  expect(onApprove).not.toHaveBeenCalled();
  expect(onDecline).not.toHaveBeenCalled();
});
