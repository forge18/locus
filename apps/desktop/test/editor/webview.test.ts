import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("webview matrix", () => {
  it.each(["wkwebview", "webview2", "webkitgtk"])("records %s as untested when no runner exists", (platform) => {
    const matrix = JSON.parse(readFileSync(resolve(process.cwd(), "webview-matrix.json"), "utf8"));
    expect(matrix[platform].status).toBe("untested");
    expect(matrix[platform].checks).toEqual(expect.arrayContaining(["completion", "diagnostics", "mergeview"]));
  });
});
