import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { SRC } from "../css";

const dataDir = resolve(SRC, "data");
const modules = readdirSync(dataDir)
  .filter((f) => f.endsWith(".ts"))
  .sort();

/** Every fixture module a screen reads has to be reachable through an accessor. */
const DATA_SETS = [
  "agent-defs",
  "artifacts",
  "board",
  "core",
  "develop",
  "extensions",
  "guardrails",
  "harnesses",
  "inbox",
  "plan",
  "runs",
  "sessions",
  "settings",
  "status",
  "strip",
  "telemetry",
  "wiki",
  "workflow",
];

describe("data/accessors", () => {
  it("has one accessor module per data set", () => {
    expect(modules.map((f) => f.replace(/\.ts$/, ""))).toEqual(DATA_SETS);
  });

  it("returns data from every accessor", async () => {
    const results: Record<string, unknown> = {};
    for (const file of modules) {
      const name = file.replace(/\.ts$/, "");
      const mod = (await import(`../../src/data/${name}.ts`)) as Record<
        string,
        unknown
      >;
      for (const [key, value] of Object.entries(mod)) {
        if (typeof value !== "function" || !key.startsWith("use")) continue;
        // Accessors that need an id get one that exists in the fixtures.
        const arg =
          key === "useEvidence"
            ? "t-010"
            : key === "useArtifactComments"
              ? "a-1"
              : "s-0000";
        results[`${name}.${key}`] = (value as (a?: string) => unknown)(arg);
      }
    }
    expect(Object.keys(results).length).toBeGreaterThanOrEqual(
      DATA_SETS.length,
    );
    for (const [name, value] of Object.entries(results)) {
      expect(value, `${name} returned nothing`).not.toBe(undefined);
    }
  });

  it("names the command each accessor becomes, so the M1 swap has a target", () => {
    for (const file of modules) {
      const source = readFileSync(resolve(dataDir, file), "utf8");
      const accessors = source.match(/^export function \w+/gm) ?? [];
      const becomes = source.match(/Becomes: /g) ?? [];
      expect(
        becomes.length,
        `${file}: ${accessors.length} accessors, ${becomes.length} targets`,
      ).toBe(accessors.length);
    }
  });

  it("is the only thing that reads a fixture", () => {
    for (const file of modules) {
      const source = readFileSync(resolve(dataDir, file), "utf8");
      expect(source, `${file} reads no fixture`).toMatch(
        /from '\.\.\/(fixtures|types)\//,
      );
    }
  });

  it("hands back the same objects the fixtures hold, without copying", async () => {
    const { useSessions } = await import("../../src/data/sessions");
    const { SESSIONS } = await import("../../src/fixtures/sessions");
    expect(useSessions()).toBe(SESSIONS);
  });

  it("reports the computed harness summary rather than a written-down number", async () => {
    const { useHarnessSummary } = await import("../../src/data/harnesses");
    expect(useHarnessSummary()).toEqual({
      harnesses: 11,
      entries: 88,
      downgrades: 29,
    });
  });
});
