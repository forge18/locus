import { describe, expect, it } from "vitest";
import { editorKeymap, editorKeymapBindings } from "../../src/editor/keymap";

describe("editor/one-keymap", () => {
  it("exports one configured keymap", () => {
    expect(editorKeymap).toBeDefined();
    expect(editorKeymapBindings.length).toBeGreaterThan(0);
  });
});
