import { describe, expect, it } from "vitest";
import { SESSIONS, eventsFor } from "../../src/fixtures/sessions";
import { RUN_ROWS } from "../../src/fixtures/runs";
import { SPEND } from "../../src/fixtures/telemetry";

describe("fixtures/usage-unknown-exists", () => {
  it("has at least one session whose usage is unknown", () => {
    expect(SESSIONS.filter((s) => s.usage === null).length).toBeGreaterThan(0);
  });

  it("never fakes a zero — an unreported spend is null, not 0", () => {
    for (const s of SESSIONS) {
      if (s.usage === null) continue;
      expect(s.usage.input + s.usage.output, s.id).toBeGreaterThan(0);
    }
    for (const r of RUN_ROWS) {
      expect(r.tokens === null || r.tokens > 0, r.id).toBe(true);
    }
  });

  it("carries the unknown through to the rows a screen renders", () => {
    expect(
      SPEND.some((r) => r.tokens === null && r.cacheReadPct === null),
    ).toBe(true);
  });

  it("passes the session usage through to the events that carry it", () => {
    const unknown = SESSIONS.find((s) => s.usage === null)!;
    const carrying = eventsFor(unknown.id).filter(
      (e) => e.verb === "assistant",
    );
    expect(carrying.length).toBeGreaterThan(0);
    for (const e of carrying) expect(e.usage).toBeNull();
  });
});
