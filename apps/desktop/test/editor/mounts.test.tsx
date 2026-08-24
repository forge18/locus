import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { EditorPane } from "../../src/editor/EditorPane";
import { editorThemeTokens } from "../../src/editor/theme";

const file = { uri: "file:///workspace/app.ts", path: "app.ts", languageId: "typescript", content: "const answer = 42;" };
const language = { id: "typescript", extensions: [".ts"], grammar: "typescript" as const };

describe("editor mounts", () => {
  it("mounts CodeMirror in the side pane with token-based theme", () => {
    const { getByTestId } = render(() => <EditorPane file={file} language={language} />);
    expect(getByTestId("editor-surface").querySelector(".cm-editor")).toBeTruthy();
    expect(editorThemeTokens).toContain("--surface-ground");
  });
});
