import { describe, expect, it } from "vitest";
import { fetchRunningCount } from "../../src/data/strip";
import { configureProjectsStub } from "../projects/provider-stub";

describe("shell/running-count", () => {
  it("returns the live running count", async () => {
    // The agents-only rule is pinned host-side (shell_queries
    // only_running_runs_count); the accessor relays the number.
    configureProjectsStub({ runningCount: 5 });
    const envelope = await fetchRunningCount();
    expect(envelope).toEqual({ status: "ready", data: 5 });
  });

  it("relays a failed read as a typed failure", async () => {
    configureProjectsStub({ fail: ["running_count"] });
    const envelope = await fetchRunningCount();
    expect(envelope).toEqual({
      status: "failed",
      error: { command: "running_count", message: "IPC failure for running_count" },
    });
  });
});
