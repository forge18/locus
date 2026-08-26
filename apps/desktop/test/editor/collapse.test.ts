import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("editor/collapse", () => {
  it("configures collapsed unchanged chunks and handoff styling", () => {
    const source = readFileSync(resolve(process.cwd(), "src/editor/MergeEditor.tsx"), "utf8");
    expect(source).toContain("collapseUnchanged");
    expect(source).toContain("locus-merge-revert");
  });
});
