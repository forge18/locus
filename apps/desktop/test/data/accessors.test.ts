import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { SRC } from "../css";
import type { Envelope } from "../../src/data/envelope";
import { configureDataProvider } from "../../src/data/provider";
import { configureDemoProvider } from "../../src/data/demo/bootstrap";

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
  "dispatch",
  "extensions",
  "guardrails",
  "harnesses",
  "knowledge",
  "mail",
  "plan",
  "qa",
  "settings",
  "telemetry",
  "workflow",
  "workflow-events",
];
const NON_FIXTURE_DATA_SETS = ["bots", "work-items"];
/** Slices that migrated to the live provider leave the fixture list here. */
const LIVE_DATA_SETS = ["core", "inbox", "runs", "sessions", "strip"];

/** The task-2 seam: typed envelope + provider. Not data sets, never fixture-backed. */
const SEAM_MODULES = ["envelope", "provider"];
const DATA_SETS = [
  ...FIXTURE_DATA_SETS,
  ...NON_FIXTURE_DATA_SETS,
  ...LIVE_DATA_SETS,
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

  it("keeps fixture reads behind the explicit demo boundary", () => {
    for (const name of FIXTURE_DATA_SETS) {
      const file = `${name}.ts`;
      const source = readFileSync(resolve(dataDir, file), "utf8");
      expect(source, `${file} imports no fixture directly`).not.toMatch(
        /from ["']\.\.\/fixtures\//,
      );
    }

    const demo = readFileSync(
      resolve(dataDir, "demo/demo-provider.ts"),
      "utf8",
    );
    expect(demo).toMatch(/from ["']\.\.\/\.\.\/fixtures\//);
  });

  it("keeps the normalized event literal in fixtures, re-exported by data", async () => {
    const fixture = await import("../../src/fixtures/workflow-events");
    const data = await import("../../src/data/workflow-events");

    expect(data.WORKFLOW_EVENTS).toBe(fixture.WORKFLOW_EVENTS);
    expect(data.workflowEventsForTranscript()).toBe(fixture.WORKFLOW_EVENTS);
  });

  it("pages sessions through the live provider with the project scope", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const rows = [{ id: "s-0000", project: "tapestry", agent: "builder" }];
    configureDataProvider({
      kind: "demo",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: rows as T[] };
      },
      async queryOne<T>(command: string) {
        calls.push({ command });
        return { status: "empty" } as Envelope<T>;
      },
    });

    const { fetchSessions } = await import("../../src/data/sessions");
    const envelope = await fetchSessions("p-tapestry", 0, 50);
    expect(envelope.status).toBe("ready");
    expect(calls).toEqual([
      {
        command: "sessions_list",
        args: { projectId: "p-tapestry", offset: 0, limit: 50 },
      },
    ]);
  });

  it("pages plans through the live provider with the project scope", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    configureDataProvider({
      kind: "demo",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "empty" } as Envelope<T[]>;
      },
      async queryOne<T>() {
        return {
          status: "failed",
          error: { command: "unexpected", message: "no" },
        } as Envelope<T>;
      },
    });

    const { fetchPlans } = await import("../../src/data/plan");
    const envelope = await fetchPlans("p-tapestry");
    expect(envelope.status).toBe("empty");
    expect(calls).toEqual([
      { command: "plans_list", args: { projectId: "p-tapestry" } },
    ]);
  });

  it("scopes a session's runs to that session through the live provider", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    configureDataProvider({
      kind: "demo",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "empty" } as Envelope<T[]>;
      },
      async queryOne<T>() {
        return {
          status: "failed",
          error: { command: "unexpected", message: "no" },
        } as Envelope<T>;
      },
    });

    const { fetchRunsForSession } = await import("../../src/data/sessions");
    const envelope = await fetchRunsForSession("s-0000");
    expect(envelope.status).toBe("empty");
    expect(calls).toEqual([
      { command: "runs_for_session", args: { sessionId: "s-0000" } },
    ]);
  });

  it("reports the computed harness summary rather than a written-down number", async () => {
    configureDemoProvider();
    const { useHarnessSummary } = await import("../../src/data/harnesses");
    expect(useHarnessSummary()).toEqual({
      harnesses: 11,
      entries: 88,
      downgrades: 29,
    });
  });

  it("routes knowledge, analytics, and telemetry reads through the configured provider", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    configureDataProvider({
      kind: "demo",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return {
          status: "ready",
          data: [] as T[],
        };
      },
      async queryOne<T>() {
        return { status: "empty" } as Envelope<T>;
      },
    });

    const { fetchLongTermFacts } = await import("../../src/data/knowledge");
    const { fetchArtifactsFromCore } = await import("../../src/data/artifacts");
    const { fetchAtAGlanceMetrics } = await import("../../src/data/analytics");
    const { fetchTelemetryMetrics } = await import("../../src/data/telemetry");
    const { fetchQaFindings } = await import("../../src/data/qa");
    expect((await fetchLongTermFacts("p-tapestry")).status).toBe("ready");
    expect((await fetchArtifactsFromCore("p-tapestry")).status).toBe("ready");
    expect((await fetchAtAGlanceMetrics("tapestry", "30d")).status).toBe(
      "ready",
    );
    expect((await fetchTelemetryMetrics("tapestry", "30d")).status).toBe(
      "ready",
    );
    expect((await fetchQaFindings("p-tapestry")).status).toBe("ready");
    expect(calls).toEqual([
      { command: "memory_facts", args: { projectId: "p-tapestry" } },
      { command: "artifacts_list", args: { projectId: "p-tapestry" } },
      {
        command: "analytics_at_a_glance",
        args: { query: { scope: "tapestry", range: "30d" } },
      },
      {
        command: "telemetry_metrics",
        args: { query: { scope: "tapestry", range: "30d" } },
      },
      { command: "qa_snapshot", args: { projectId: "p-tapestry" } },
    ]);
  });
});
