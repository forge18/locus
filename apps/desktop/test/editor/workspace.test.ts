import { describe, expect, it } from "vitest";
import { MultiFileWorkspace } from "../../src/editor/lsp";
import { createLspClient } from "../../src/editor/lsp";

describe("editor/workspace", () => {
  it("provides a workspace abstraction for multiple files", () => {
    const workspace = new MultiFileWorkspace(createLspClient("file:///workspace"));
    expect(workspace.files).toEqual([]);
    expect(workspace.syncFiles()).toEqual([]);
  });
});
