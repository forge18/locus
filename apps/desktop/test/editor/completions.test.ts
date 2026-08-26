import { describe, expect, it } from "vitest";
import { createLspClient, languageExtensions } from "../../src/editor/lsp";

describe("editor/completions", () => {
  it("enables server completion, hover, and signature help", () => {
    const client = createLspClient("file:///workspace");
    expect(languageExtensions({ id: "typescript", extensions: [".ts"], grammar: "typescript" }, client, "file:///workspace/a.ts")).toHaveLength(5);
    client.disconnect();
  });
});
