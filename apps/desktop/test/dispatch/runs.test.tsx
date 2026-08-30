import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { DispatchView } from "../../src/screens/dispatch/DispatchView";
import { configureProjectsStub } from "../projects/provider-stub";

describe("dispatch runs", () => {
  it("renders the live run count once the page lands", async () => {
    configureProjectsStub({
      runsPage: [
        {
          id: "run-1",
          project: "tapestry",
          agent: "builder",
          branch: "agent/tapestry",
          status: "completed",
          harness: "claude",
          role: "builder",
          model: "claude-opus-4",
          events: 3,
          errors: 1,
          startedAt: "2026-08-30T12:00:00Z",
        },
      ],
    });
    const { getByTestId } = render(() => <DispatchView tab="runs" />);

    await waitFor(() =>
      expect(getByTestId("dispatch-pause-controls").textContent).toContain(
        "1 runs",
      ),
    );
    expect(getByTestId("dispatch-runs-table").textContent).toContain("run-1");
  });

  it("surfaces an IPC failure instead of the old fixture rows", async () => {
    configureProjectsStub({ fail: ["dispatch_runs_page"] });
    const { getByTestId } = render(() => <DispatchView tab="runs" />);
    await waitFor(() =>
      expect(getByTestId("dispatch-runs-table").textContent).toContain(
        "IPC failure for dispatch_runs_page",
      ),
    );
  });
});
