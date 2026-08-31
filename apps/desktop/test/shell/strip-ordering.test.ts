import { describe, expect, it } from "vitest";
import { fetchStripCards } from "../../src/data/strip";
import { configureProjectsStub } from "../projects/provider-stub";

const now = Math.floor(Date.now() / 1000);

/** Seed rows whose fixture order deliberately disagrees with the sort order. */
function seedOutOfOrder() {
  configureProjectsStub({
    stripCards: [
      // newest, running — activity would put it first
      { id: "run-a", project: "alpha", agent: "builder", status: "running", startedEpoch: now },
      // stuck — attention puts it first despite being the least recently active
      { id: "run-b", project: "beta", agent: "builder", status: "stuck", startedEpoch: now - 600 },
      { id: "run-c", project: "gamma", agent: "reviewer", status: "running", startedEpoch: now - 60 },
    ],
  });
}

describe("shell/strip-ordering", () => {
  it("puts the stuck card first even though it is the least recently active", async () => {
    seedOutOfOrder();
    const envelope = await fetchStripCards();
    const ids = envelope.status === "ready" ? envelope.data.map((c) => c.id) : [];
    // Stuck first; the two running cards tie-break by activity, most recent first.
    expect(ids).toEqual(["run-b", "run-a", "run-c"]);
  });

  it("breaks ties by activity, most recent first", async () => {
    configureProjectsStub({
      stripCards: [
        { id: "run-late", project: "alpha", agent: "builder", status: "running", startedEpoch: now - 30 },
        { id: "run-early", project: "beta", agent: "builder", status: "running", startedEpoch: now - 300 },
      ],
    });
    const envelope = await fetchStripCards();
    const ids = envelope.status === "ready" ? envelope.data.map((c) => c.id) : [];
    expect(ids).toEqual(["run-late", "run-early"]);
  });

  it("derives elapsed minutes from the run's started epoch", async () => {
    configureProjectsStub({
      stripCards: [
        { id: "run-1", project: "alpha", agent: "builder", status: "running", startedEpoch: now - 180 },
      ],
    });
    const envelope = await fetchStripCards();
    const card = envelope.status === "ready" ? envelope.data[0] : undefined;
    expect(card?.idleMinutes).toBe(3);
  });

  it("passes a failed read through as a typed failure", async () => {
    configureProjectsStub({ fail: ["strip_cards"] });
    const envelope = await fetchStripCards();
    expect(envelope).toEqual({
      status: "failed",
      error: { command: "strip_cards", message: "IPC failure for strip_cards" },
    });
  });
});
