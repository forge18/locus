import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import {
  applySemanticTokenDelta,
  applySemanticTokens,
  decodeSemanticTokens,
  semanticTokensExtension,
} from "../../src/editor/semanticTokens";

describe("editor/semantic-tokens", () => {
  it("decodes relative full responses and applies deltas", () => {
    const initial = [0, 0, 3, 1, 0, 1, 2, 4, 2, 0];
    expect(decodeSemanticTokens(initial)).toEqual([
      { line: 0, start: 0, length: 3, tokenType: 1, modifiers: 0 },
      { line: 1, start: 2, length: 4, tokenType: 2, modifiers: 0 },
    ]);
    expect(
      applySemanticTokenDelta(initial, [
        { start: 0, deleteCount: 5, data: [0, 0, 3, 0, 4] },
      ]),
    ).toEqual([0, 0, 3, 0, 4, 1, 2, 4, 2, 0]);
  });

  it("renders supported tokens as CodeMirror decorations", () => {
    const parent = document.createElement("div");
    const view = new EditorView({
      state: EditorState.create({
        doc: "const value = 1",
        extensions: [semanticTokensExtension()],
      }),
      parent,
    });
    applySemanticTokens(view, [
      { line: 0, start: 0, length: 5, tokenType: 3, modifiers: 0 },
    ]);
    expect(parent.querySelector(".cm-lsp-semantic-token-3")?.textContent).toBe(
      "const",
    );
    view.destroy();
  });

  it("rejects malformed or out-of-bounds deltas", () => {
    expect(() => decodeSemanticTokens([0, 0])).toThrow("five integers");
    expect(() =>
      applySemanticTokenDelta(
        [0, 0, 1, 0, 0],
        [{ start: 4, deleteCount: 2, data: [] }],
      ),
    ).toThrow("out of bounds");
  });
});
