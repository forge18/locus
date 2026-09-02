import { render, waitFor } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { EditorView } from "@codemirror/view";
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

  it("updates the mounted editor when the selected file changes", async () => {
    const changes: string[] = [];
    const [selectedFile, setSelectedFile] = createSignal(
      file("const first = true;"),
    );
    const view = render(() => (
      <EditorPane
        file={selectedFile()}
        language={language}
        onChange={(content) => changes.push(content)}
      />
    ));

    expect(view.getByTestId("editor-surface").textContent).toContain(
      "const first = true;",
    );
    setSelectedFile(file("const second = false;"));

    await waitFor(() =>
      expect(view.getByTestId("editor-surface").textContent).toContain(
        "const second = false;",
      ),
    );
    expect(view.getByTestId("editor-surface").textContent).not.toContain(
      "const first = true;",
    );
    expect(changes).toEqual([]);
    view.unmount();
  });

  it("reattaches LSP and replaces content when the file identity changes", async () => {
    mocks.attachTauriLsp
      .mockRejectedValueOnce(new Error("LSP unavailable"))
      .mockRejectedValueOnce(new Error("LSP unavailable"));
    const [selectedFile, setSelectedFile] = createSignal(
      file("const first = true;"),
    );
    const view = render(() => (
      <EditorPane
        file={selectedFile()}
        language={language}
        projectRoot="/workspace"
        paneId="pane-1"
      />
    ));
    await waitFor(() =>
      expect(view.getByTestId("editor-surface").getAttribute("data-state")).toBe(
        "error",
      ),
    );

    setSelectedFile({
      ...file("const second = false;"),
      uri: "file:///workspace/other.ts",
      path: "other.ts",
    });
    await waitFor(() =>
      expect(mocks.attachTauriLsp).toHaveBeenCalledTimes(2),
    );
    await waitFor(() =>
      expect(view.getByTestId("editor-surface").textContent).toContain(
        "const second = false;",
      ),
    );
    expect(mocks.attachTauriLsp).toHaveBeenLastCalledWith(
      expect.objectContaining({ filePath: "other.ts" }),
    );
    view.unmount();
  });

  it("shows loading, then keeps the real file editable when LSP setup fails", async () => {
    const changes: string[] = [];
    mocks.attachTauriLsp.mockRejectedValueOnce(new Error("LSP unavailable"));
    const view = render(() => (
      <EditorPane
        file={file("const actual = true;")}
        language={language}
        projectRoot="/workspace"
        paneId="pane-1"
        onChange={(content) => changes.push(content)}
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
    const content = view
      .getByTestId("editor-surface")
      .querySelector(".cm-content");
    expect(content?.textContent).toContain("const actual = true;");
    const editor = content
      ? EditorView.findFromDOM(content as HTMLElement)
      : undefined;
    expect(editor).toBeTruthy();
    if (editor) {
      editor.dispatch({
        changes: {
          from: 0,
          to: editor.state.doc.length,
          insert: "const edited = true;",
        },
      });
    }
    expect(changes[changes.length - 1]).toBe("const edited = true;");
    view.unmount();
  });
});
