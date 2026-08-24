import { describe, expect, it } from "vitest";
import { EditorPane } from "../../src/editor/EditorPane";
import { FullWindowEditor } from "../../src/editor/FullWindowEditor";

describe("editor/two-zoom-levels", () => {
  it("provides both zoom levels", () => {
    expect(EditorPane).toBeTypeOf("function");
    expect(FullWindowEditor).toBeTypeOf("function");
  });
});
