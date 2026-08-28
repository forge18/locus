import { expect, it } from "vitest";
import { workspacePath } from "../../src/panes/agent-pane-utils";

it("agent-panel/safe-paths rejects traversal and normalizes the container workspace", () => {
  expect(workspacePath("/workspace/src/lib.rs")).toBe("src/lib.rs");
  expect(workspacePath("../secrets.env")).toBeUndefined();
  expect(workspacePath("/etc/passwd")).toBeUndefined();
});
