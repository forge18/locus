import { describe, expect, it } from "vitest";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";
import {
  HARNESSES,
  HARNESS_COUNT,
} from "../../src/fixtures/generated/harnesses";
import { SRC } from "../css";

const harnessDir = resolve(SRC, "../../../harnesses");

/** Mirror the core registry: harnesses/*.toml plus one level of plugin subdirectories. */
function tomlFiles(): string[] {
  const direct = readdirSync(harnessDir).filter((f) => f.endsWith(".toml"));
  const nested = readdirSync(harnessDir, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .flatMap((e) =>
      readdirSync(resolve(harnessDir, e.name)).filter((f) =>
        f.endsWith(".toml"),
      ),
    );
  return [...direct, ...nested];
}

describe("fixtures/harness-count", () => {
  it("reports 11 harnesses", () => {
    expect(HARNESS_COUNT).toBe(11);
  });

  it("counts one per harnesses/*.toml, so adding a file moves the number", () => {
    const files = tomlFiles();
    expect(HARNESS_COUNT).toBe(files.length);
    expect(HARNESSES.length).toBe(files.length);
  });

  it("names each harness by the name inside its file, not its filename", () => {
    expect(HARNESSES.map((h) => h.name).sort()).toEqual([
      "aider",
      "antigravity",
      "claude",
      "codex",
      "copilot",
      "cursor",
      "dsh",
      "gemini",
      "omp",
      "opencode",
      "pi",
    ]);
  });

  it("is sorted, so regenerating never churns the diff", () => {
    const names = HARNESSES.map((h) => h.name);
    expect(names).toEqual([...names].sort());
  });
});
