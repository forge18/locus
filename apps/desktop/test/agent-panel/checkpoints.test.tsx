import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { checkpoints as checkpointList, mount, session } from "./support";

it("agent-panel/checkpoints restores and undoes without removing the stream", async () => {
  const onRestore = vi.fn();
  const onUndo = vi.fn();
  const view = mount(<AgentPane runId="run-1" live={false} session={session} checkpoints={checkpointList} onRestoreCheckpoint={onRestore} onUndoCheckpoint={onUndo} />);
  await fireEvent.click(view.getByRole("button", { name: "Restore" }));
  expect(onRestore).toHaveBeenCalledWith(checkpointList[0]);
  expect(view.getByRole("status").textContent).toContain("transcript remains intact");
  await fireEvent.click(view.getByRole("button", { name: "Undo" }));
  expect(onUndo).toHaveBeenCalledWith(checkpointList[0]);
});
