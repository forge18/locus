import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("editor/shared-by-import", () => {
  it("asserts both zoom wrappers import the shared surface", () => {
    for (const name of ["EditorPane.tsx", "FullWindowEditor.tsx"]) {
      const source = readFileSync(resolve(process.cwd(), `src/editor/${name}`), "utf8");
      expect(source).toContain('from "./EditorSurface"');
    }
  });
});
