import { describe, expect, it } from "vitest";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";
import { SRC } from "../css";

/** One module per screen that shows data, plus the two computed sets. */
const AUTHORED = [
  "agent-defs",
  "analytics",
  "artifacts",
  "board",
  "core",
  "dispatch",
  "extensions",
  "inbox",
  "knowledge",
  "mail",
  "plan",
  "qa",
  "runs",
  "sessions",
  "settings",
  "settings-guardrails",
  "strip",
  "telemetry",
  "workflow",
];

const files = readdirSync(resolve(SRC, "fixtures"))
  .filter((f) => f.endsWith(".ts"))
  .map((f) => f.replace(/\.ts$/, ""));

describe("fixtures/all-present", () => {
  it("has a module for every screen that shows data", () => {
    for (const name of AUTHORED) {
      expect(files, `missing src/fixtures/${name}.ts`).toContain(name);
    }
  });

  it("has the computed harness set alongside the authored ones", () => {
    expect(
      readdirSync(resolve(SRC, "fixtures/generated")).filter((f) =>
        f.endsWith(".ts"),
      ),
    ).toContain("harnesses.ts");
  });

  it("carries data in every authored module", async () => {
    for (const name of AUTHORED) {
      const mod = (await import(`../../src/fixtures/${name}.ts`)) as Record<
        string,
        unknown
      >;
      const exports = Object.entries(mod).filter(([k]) => k !== "default");
      expect(exports.length, `${name} exports nothing`).toBeGreaterThan(0);

      const hasData = exports.some(
        ([, v]) =>
          (Array.isArray(v) && v.length > 0) ||
          (typeof v === "object" && v !== null),
      );
      expect(hasData, `${name} exports no data`).toBe(true);
    }
  });

  it("keeps generated ids unique, ordered, and joined to what they reference", async () => {
    const { SESSIONS, RUNS } = await import("../../src/fixtures/sessions");
    const { RUN_ROWS } = await import("../../src/fixtures/runs");
    const { PROJECTS, REPOS } = await import("../../src/fixtures/core");

    const sessionIds = SESSIONS.map((session) => session.id);
    const runIds = RUNS.map((run) => run.id);

    // Unique and ordered, so id lookups and list windows stay deterministic.
    expect(new Set(sessionIds).size).toBe(sessionIds.length);
    expect(new Set(runIds).size).toBe(runIds.length);
    expect(new Set(RUN_ROWS.map((run) => run.id)).size).toBe(RUN_ROWS.length);
    expect(sessionIds).toEqual([...sessionIds].sort());
    expect(runIds).toEqual([...runIds].sort());
    expect(RUN_ROWS.map((run) => run.id)).toEqual(
      RUN_ROWS.map((run) => run.id).sort(),
    );

    // Every generated session resolves against the core fixtures it joins.
    const projectIds = new Set(PROJECTS.map((project) => project.id));
    const repos = new Map(REPOS.map((repo) => [repo.id, repo]));
    for (const session of SESSIONS) {
      expect(
        projectIds.has(session.projectId),
        `${session.id} names the unknown project ${session.projectId}`,
      ).toBe(true);
      const repo = repos.get(session.repoId);
      expect(
        repo,
        `${session.id} names the unknown repo ${session.repoId}`,
      ).toBeDefined();
      expect(
        repo!.projectId,
        `${session.id} pairs ${session.repoId} with the wrong project`,
      ).toBe(session.projectId);
      expect(
        session.handedOffFrom === null ||
          sessionIds.includes(session.handedOffFrom),
        `${session.id} hands off to a session that does not exist`,
      ).toBe(true);
    }

    // The session/run join is exact in both directions.
    for (const session of SESSIONS) {
      expect(session.runIds.length, `${session.id} names no run`).toBeGreaterThan(
        0,
      );
      for (const runId of session.runIds) {
        const run = RUNS.find((candidate) => candidate.id === runId);
        expect(
          run,
          `${session.id} names the missing run ${runId}`,
        ).toBeDefined();
        expect(
          run!.sessionId,
          `${runId} belongs to another session`,
        ).toBe(session.id);
      }
    }
    for (const run of RUNS) {
      expect(
        sessionIds.includes(run.sessionId),
        `${run.id} belongs to the missing session ${run.sessionId}`,
      ).toBe(true);
    }
  });

  it("preserves the usage-unknown contract in generated rows", async () => {
    const { SESSIONS } = await import("../../src/fixtures/sessions");
    const { ALL_SESSION_ROWS } = await import("../../src/fixtures/telemetry");

    expect(SESSIONS.some((session) => session.usage === null)).toBe(true);
    expect(ALL_SESSION_ROWS.some((session) => session.tokens === null)).toBe(true);
  });
});
