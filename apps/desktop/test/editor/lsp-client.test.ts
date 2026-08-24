import { describe, expect, it } from "vitest";
import { createLspClient, supervisorTransport } from "../../src/editor/lsp";

describe("editor/lsp-client", () => {
  it("adapts the host LSP supervisor to CodeMirror transport", () => {
    const sent: string[] = [];
    const handlers = new Set<(message: string) => void>();
    const transport = supervisorTransport({
      send: (message) => sent.push(message),
      subscribe: (handler) => void handlers.add(handler),
      unsubscribe: (handler) => void handlers.delete(handler),
    });
    transport.send("{\"jsonrpc\":\"2.0\"}");
    expect(sent).toHaveLength(1);
    expect(handlers).toHaveLength(0);
  });
  it("creates a client rooted at the editor clone", () => {
    expect(createLspClient("file:///workspace").connected).toBe(false);
  });
});
