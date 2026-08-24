import { describe, expect, it } from "vitest";
import { editorKeymapBindings } from "../../src/editor/keymap";
import { formatKeymap, renameKeymap } from "@codemirror/lsp-client";

describe("editor/rename-format", () => {
  it("includes server rename and format bindings", () => {
    expect(editorKeymapBindings).toEqual(expect.arrayContaining([...formatKeymap, ...renameKeymap]));
  });
});
