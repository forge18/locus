import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("develop/search-opens-at-line", () => {
  it("passes the selected result to the editor callback with its line and column", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/screens/develop/SearchView.tsx"),
      "utf8",
    );
    expect(source).toContain("props.onOpenResult?.(");
    expect(source).toContain("data-line={result.line}");
    expect(source).toContain("result.column");
  });
});
