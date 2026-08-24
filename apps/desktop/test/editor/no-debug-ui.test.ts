import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("editor/no-debug-ui", () => {
  it("does not add debug controls to the editor surface", () => {
    const source = readFileSync(resolve(process.cwd(), "src/editor/EditorSurface.tsx"), "utf8");
    expect(source).not.toMatch(/debug gutter|variables pane|step control/i);
    expect(source).not.toContain("debugGutter");
  });
});
