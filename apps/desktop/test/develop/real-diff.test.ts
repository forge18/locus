import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("develop/real-diff", () => {
  it("uses MergeEditor rather than a fixture diff", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/screens/develop/DevelopView.tsx"),
      "utf8",
    );
    expect(source).toContain("MergeEditor");
    expect(source).not.toContain("diff-headers");
  });
});
