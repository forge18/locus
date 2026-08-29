import { render, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ attachTauriLsp: vi.fn() }));

vi.mock("../../src/editor/tauriLsp", () => ({
  attachTauriLsp: mocks.attachTauriLsp,
}));

import { EditorPane } from "../../src/editor/EditorPane";

const language = {
  id: "typescript",
  extensions: [".ts"],
  grammar: "typescript" as const,
};

const file = (content: string) => ({
  uri: "file:///workspace/app.ts",
  path: "app.ts",
  languageId: "typescript",
  content,
});

describe("editor states", () => {
  beforeEach(() => mocks.attachTauriLsp.mockReset());

  it("marks an empty file separately from a loaded file", () => {
    const empty = render(() => <EditorPane file={file("")} language={language} />);
    expect(empty.getByTestId("editor-surface").getAttribute("data-state")).toBe(
      "empty",
    );
    expect(empty.getByTestId("editor-empty")).toBeTruthy();
    empty.unmount();

    const loaded = render(() => (
      <EditorPane file={file("const answer = 42;")} language={language} />
    ));
    expect(loaded.getByTestId("editor-surface").getAttribute("data-state")).toBe(
      "loaded",
    );
    loaded.unmount();
  });

  it("shows loading, then keeps the real file editable when LSP setup fails", async () => {
    mocks.attachTauriLsp.mockRejectedValueOnce(new Error("LSP unavailable"));
    const view = render(() => (
      <EditorPane
        file={file("const actual = true;")}
        language={language}
        projectRoot="/workspace"
        paneId="pane-1"
      />
    ));

    expect(view.getByTestId("editor-surface").getAttribute("data-state")).toBe(
      "loading",
    );
    expect(view.getByTestId("editor-loading")).toBeTruthy();

    await waitFor(() =>
      expect(view.getByTestId("editor-surface").getAttribute("data-state")).toBe(
        "error",
      ),
    );
    expect(view.getByTestId("inline-error-cause").textContent).toBe(
      "LSP unavailable",
    );
    expect(view.getByTestId("editor-surface").querySelector(".cm-content")?.textContent).toContain(
      "const actual = true;",
    );
    view.unmount();
  });
});
