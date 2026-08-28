import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/file-link delegates a safe path to the editor pane", async () => {
  const onOpenFile = vi.fn();
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("assistant", 0, { raw: { path: "src/file.ts" } })]} onOpenFile={onOpenFile} />);
  const link = view.getByTestId("agent-turn").querySelector("[data-file-path]") as HTMLElement;
  expect(link.getAttribute("data-open-pane")).toBe("editor");
  await fireEvent.click(link);
  expect(onOpenFile).toHaveBeenCalledWith("src/file.ts");
});
