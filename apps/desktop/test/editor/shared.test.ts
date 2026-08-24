import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { editorKeymapBindings } from "../../src/editor/keymap";
import { EditorPane } from "../../src/editor/EditorPane";
import { FullWindowEditor } from "../../src/editor/FullWindowEditor";

describe("editor zoom levels", () => {
  it("uses one keymap module", () => {
    expect(editorKeymapBindings.length).toBeGreaterThan(0);
    expect(readFileSync(resolve(process.cwd(), "src/editor/EditorPane.tsx"), "utf8")).toContain("./EditorSurface");
    expect(readFileSync(resolve(process.cwd(), "src/editor/FullWindowEditor.tsx"), "utf8")).toContain("./EditorSurface");
  });
  it("exposes pane and full-window components", () => {
    expect(EditorPane).toBeDefined();
    expect(FullWindowEditor).toBeDefined();
  });
});
