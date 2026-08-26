import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("develop/git-from-core", () => {
  it("renders the repo_git_state seam rather than a git fixture", () => {
    const source = readFileSync(
      resolve(process.cwd(), "src/screens/develop/DevelopView.tsx"),
      "utf8",
    );
    expect(source).toContain("repo_git_state");
    expect(source).toContain("gitState");
  });
});
