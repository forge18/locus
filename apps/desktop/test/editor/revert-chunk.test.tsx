import { describe, expect, it } from "vitest";
import { createMergeView } from "../../src/editor/MergeEditor";

describe("editor/revert-chunk", () => {
  it("provides a per-chunk revert control", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const merge = createMergeView(host, "one\ntwo\n", "one\nchanged\n");
    await new Promise((resolve) => setTimeout(resolve, 20));
    const button = host.querySelector<HTMLButtonElement>(".locus-merge-revert");
    expect(button).toBeTruthy();
    expect(button?.getAttribute("aria-label")).toBe(
      "Revert this chunk into the base",
    );
    button?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(merge.a.state.doc.toString()).toContain("changed");
    merge.destroy();
  });
});
