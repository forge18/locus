import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { SRC } from "../css";

const dataDir = resolve(SRC, "data");
const modules = readdirSync(dataDir)
  .filter((f) => f.endsWith(".ts"))
  .sort();

/** Every fixture module a screen reads has to be reachable through an accessor. */
const FIXTURE_DATA_SETS = [
  "agent-defs",
  "analytics",
  "artifacts",
  "board",
  "core",
  "dispatch",
  "extensions",
  "guardrails",
  "harnesses",
  "inbox",
  "knowledge",
  "mail",
  "plan",
  "qa",
  "runs",
  "sessions",
  "settings",
  "strip",
  "telemetry",
  "workflow",
  "workflow-events",
];
const NON_FIXTURE_DATA_SETS = ["bots", "work-items"];

/** The task-2 seam: typed envelope + provider. Not data sets, never fixture-backed. */
const SEAM_MODULES = ["envelope", "provider"];
const DATA_SETS = [
  ...FIXTURE_DATA_SETS,
  ...NON_FIXTURE_DATA_SETS,
  ...SEAM_MODULES,
]
  .map((name) => `${name}.ts`)
  .sort()
  .map((file) => file.slice(0, -3));

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
      FIXTURE_DATA_SETS.length,
    );
    for (const [name, value] of Object.entries(results)) {
      expect(value, `${name} returned nothing`).not.toBe(undefined);
    }
  }, 30_000);

  it("names the command each accessor becomes, so the M1 swap has a target", () => {
    for (const file of modules) {
      const lines = readFileSync(resolve(dataDir, file), "utf8").split("\n");
      for (const [index, line] of lines.entries()) {
        if (!/^export function use\\w+/.test(line)) continue;
        const doc = lines.slice(Math.max(0, index - 20), index).join("\\n");
        expect(doc, `${file}:${index + 1} has no command target`).toContain(
          "Becomes:",
        );
      }
    }
  });

  it("is the only thing that reads a fixture", () => {
    for (const name of FIXTURE_DATA_SETS) {
      const file = `${name}.ts`;
      const source = readFileSync(resolve(dataDir, file), "utf8");
      expect(source, `${file} reads no fixture`).toMatch(
        /from ["']\.\.\/(fixtures|types)\//,
      );
    }
  });

  it("keeps the normalized event literal in fixtures, re-exported by data", async () => {
    const fixture = await import("../../src/fixtures/workflow-events");
    const data = await import("../../src/data/workflow-events");

    expect(data.WORKFLOW_EVENTS).toBe(fixture.WORKFLOW_EVENTS);
    expect(data.workflowEventsForTranscript()).toBe(fixture.WORKFLOW_EVENTS);
  });

  it("sorts session details needs-attention first, then by idle time", async () => {
    const { useSessionDetails } = await import("../../src/data/sessions");

    // stuck, then waiting, then idle, then the rest; ties break to the most
    // recently active. The same rule the strip sorts by.
    expect(useSessionDetails().map((detail) => detail.id)).toEqual([
      "sd-weaver", // stuck
      "sd-texere", // waiting
      "sd-loom-db", // idle
      "sd-tapestry", // running, active now
      "sd-review", // running, 2m idle
    ]);
  });

  it("misses explicitly instead of throwing, and scopes runs to one session", async () => {
    const { useSessionDetail, useRunsForSession } = await import(
      "../../src/data/sessions"
    );

    expect(useSessionDetail("sd-weaver")?.status).toBe("stuck");
    expect(useSessionDetail("sd-missing")).toBeNull();

    // A detail id is not a session id: the run lookup misses with an empty list.
    expect(useRunsForSession("sd-weaver")).toEqual([]);
    const runs = useRunsForSession("s-0000");
    expect(runs.length).toBeGreaterThan(0);
    for (const run of runs) {
      expect(run.sessionId, `${run.id} is another session's run`).toBe(
        "s-0000",
      );
    }
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
