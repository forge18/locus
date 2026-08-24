import { describe, expect, it } from "vitest";
import { editorKeymapBindings } from "../../src/editor/keymap";
import { findReferencesKeymap, jumpToDefinitionKeymap } from "@codemirror/lsp-client";

describe("editor/navigation", () => {
  it("includes definition and references bindings", () => {
    expect(editorKeymapBindings).toEqual(expect.arrayContaining([...jumpToDefinitionKeymap, ...findReferencesKeymap]));
  });
});
