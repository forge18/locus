import { describe, expect, it } from "vitest";
import { runQaCheck } from "../../src/data/qa";

describe("data/qa runQaCheck", () => {
  it("reports an unsupported attempt instead of an invented result", () => {
    // The desktop backend registers no QA command, so no check can run — for
    // any input, known or not, the attempt must say so rather than claim a
    // passed (or failed) result.
    const inputs = [
      ["tapestry", "unit-tests"],
      ["tapestry", "not-a-source"],
      ["missing-project", "linters"],
    ] as const;
    for (const [projectId, sourceId] of inputs) {
      const run = runQaCheck(projectId, sourceId);
      expect(run.status).toBe("unsupported");
      expect(run.id).toBe(`qa-${projectId}-${sourceId}`);
      expect(run.project).toBe(projectId);
      expect(run.sourceId).toBe(sourceId);
    }
  });

  it("keeps the epoch startedAt as a never-started sentinel, not a real time", () => {
    expect(runQaCheck("tapestry", "lsp").startedAt).toBe(
      new Date(0).toISOString(),
    );
  });
});
