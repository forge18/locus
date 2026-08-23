import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { parse } from "smol-toml";
import { HARNESSES } from "../../src/fixtures/generated/harnesses";
import { EVENT_VERBS } from "../../src/types/event";
import { SRC } from "../css";

const harnessDir = resolve(SRC, "../../../harnesses");

/** Mirror the core registry: harnesses/*.toml plus one level of plugin subdirectories. */
function tomlFiles(): string[] {
  const direct = readdirSync(harnessDir).filter((f) => f.endsWith(".toml"));
  const nested = readdirSync(harnessDir, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .flatMap((e) =>
      readdirSync(resolve(harnessDir, e.name))
        .filter((f) => f.endsWith(".toml"))
        .map((f) => `${e.name}/${f}`),
    );
  return [...direct, ...nested];
}
const raw = Object.fromEntries(
  tomlFiles().map((f) => {
    const t = parse(readFileSync(resolve(harnessDir, f), "utf8")) as Record<
      string,
      any
    >;
    return [t.name as string, t];
  }),
);

/** ACP is the single agent-session transport; terminals are human-only. */
const MECHANISMS = ["acp"];

describe("fixtures/harness-mechanisms", () => {
  it("badges each harness with the telemetry source from its own file", () => {
    for (const h of HARNESSES) {
      expect(h.mechanism, h.name).toBe(raw[h.name].telemetry.source);
      expect(MECHANISMS, `${h.name}: ${h.mechanism}`).toContain(h.mechanism);
    }
  });

  it("uses ACP for every registered harness", () => {
    expect([...new Set(HARNESSES.map((h) => h.mechanism))].sort()).toEqual(
      [...MECHANISMS].sort(),
    );
  });

  it("derives the model flag and whether tiers can be enumerated", () => {
    for (const h of HARNESSES) {
      expect(h.modelFlag, h.name).toBe(raw[h.name].models.flag);
      expect(h.canEnumerateModels, h.name).toBe(
        (raw[h.name].models.list_argv ?? []).length > 0,
      );
    }
  });

  it("finds the two harnesses that can enumerate their own models", () => {
    expect(
      HARNESSES.filter((h) => h.canEnumerateModels)
        .map((h) => h.name)
        .sort(),
    ).toEqual(["aider", "opencode"]);
  });

  it("carries only canonical verbs in a declared emit set", () => {
    for (const h of HARNESSES) {
      for (const verb of h.emits) {
        expect(EVENT_VERBS, `${h.name} emits ${verb}`).toContain(verb as never);
      }
    }
  });

  it("refuses a harness that claims a TUI — the generator throws on it", () => {
    // Every harness in the registry passed that gate to be here.
    for (const h of HARNESSES) {
      expect(raw[h.name].launch.tui, h.name).toBe(false);
    }
  });
});
