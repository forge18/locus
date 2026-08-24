import { describe, expect, it } from "vitest";
import { createMergeView } from "../../src/editor/MergeEditor";

describe("editor/mergeview", () => {
  it("renders an agent branch against its base", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const merge = createMergeView(host, "one\ntwo\n", "one\nchanged\n");
    expect(host.querySelector(".cm-mergeView")).toBeTruthy();
    expect(merge.chunks.length).toBeGreaterThan(0);
    merge.destroy();
  });
});
