import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/user-card expands, edits, copies, and resubmits a prompt", async () => {
  const onCopy = vi.fn();
  const onResubmit = vi.fn();
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("user", 0, { text: "inspect this" })]} onCopyPrompt={onCopy} onResubmit={onResubmit} />);
  await fireEvent.click(view.getByRole("button", { name: "Expand" }));
  await fireEvent.click(view.getByRole("button", { name: "Edit" }));
  await fireEvent.input(view.getByLabelText("Edit user prompt"), { target: { value: "inspect that" } });
  await fireEvent.click(view.getByRole("button", { name: "Copy" }));
  await fireEvent.click(view.getByRole("button", { name: "Resubmit" }));
  expect(onCopy).toHaveBeenCalledWith("inspect that");
  expect(onResubmit).toHaveBeenCalledWith(expect.anything(), "inspect that");
});
