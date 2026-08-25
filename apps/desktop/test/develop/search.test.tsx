import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("develop/search", () => {
  it("labels results with their repository and keeps the editor open callback", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/screens/develop/SearchView.tsx"),
      "utf8",
    );
    expect(source).toContain("result.repo");
    expect(source).toContain("result.path");
    expect(source).toContain("result.line");
    expect(source).toContain("onOpenResult");
  });

  it("is mounted in Develop beside the real editor", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/screens/develop/DevelopView.tsx"),
      "utf8",
    );
    expect(source).toContain("SearchView");
    expect(source).toContain("searchResults");
    expect(source).toContain("MergeEditor");
  });
});
