import { describe, expect, it } from "vitest";
import { languageServerExtensions } from "@codemirror/lsp-client";

describe("editor/diagnostics", () => {
  it("uses the LSP extension bundle that renders diagnostics", () => {
    expect(languageServerExtensions().length).toBeGreaterThan(0);
  });
});
