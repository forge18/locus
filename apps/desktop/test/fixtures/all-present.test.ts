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

  it("keeps generated list identities unique and ordered", async () => {
    const { SESSIONS } = await import("../../src/fixtures/sessions");
    const { RUN_ROWS } = await import("../../src/fixtures/runs");
    const sessionIds = SESSIONS.map((session) => session.id);
    const runIds = RUN_ROWS.map((run) => run.id);

    expect(new Set(sessionIds).size).toBe(sessionIds.length);
    expect(new Set(runIds).size).toBe(runIds.length);
    expect(sessionIds[0]).toBe("s-0000");
    expect(sessionIds[sessionIds.length - 1]).toBe("s-0299");
    expect(runIds[0]).toBe("run-0000");
    expect(runIds[runIds.length - 1]).toBe("run-0611");
  });

  it("preserves the usage-unknown contract in generated rows", async () => {
    const { SESSIONS } = await import("../../src/fixtures/sessions");
    const { ALL_SESSION_ROWS } = await import("../../src/fixtures/telemetry");

    expect(SESSIONS.some((session) => session.usage === null)).toBe(true);
    expect(ALL_SESSION_ROWS.some((session) => session.tokens === null)).toBe(true);
  });
});
