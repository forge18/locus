import { describe, expect, it } from "vitest";
import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { DispatchView } from "../../src/screens/dispatch/DispatchView";
import { configureProjectsStub } from "../projects/provider-stub";

configureProjectsStub({
  autorunStates: [
    {
      projectId: "00000000-0000-0000-0000-000000000a01",
      project: "tapestry",
      state: "suspended",
    },
    {
      projectId: "00000000-0000-0000-0000-000000000a02",
      project: "loom-db",
      state: "on",
    },
    {
      projectId: "00000000-0000-0000-0000-000000000a03",
      project: "weaver",
      state: "off",
    },
    {
      projectId: "00000000-0000-0000-0000-000000000a04",
      project: "texere",
      state: "off",
    },
    {
      projectId: "00000000-0000-0000-0000-000000000a05",
      project: "amq",
      state: "suspended",
    },
  ],
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

const mount = (tab: "autorun" | "schedules" | "runs" = "autorun") =>
  render(() => <DispatchView tab={tab} />);

describe("screens/desktop-dispatch", () => {
  it("renders autorun as a per-project switch from demo states", async () => {
    const { getByTestId } = mount();

    await waitFor(() =>
      expect(
        getByTestId("autorun-projects").querySelectorAll("[data-project]")
          .length,
      ).toBe(5),
    );
    expect(getByTestId("dispatch-autorun").textContent).toContain(
      "Autorun is on or off, per project",
    );
    expect(
      getByTestId("autorun-project-weaver").getAttribute("data-state"),
    ).toBe("suspended");
    expect(
      getByTestId("autorun-project-loom-db").getAttribute("data-state"),
    ).toBe("on");
  });

  it("surfaces stop-all scope, handoff preservation, and its ten-minute restore window", async () => {
    const { getByRole, getByTestId, queryByTestId } = mount();

    expect(queryByTestId("stop-all-dialog")).toBeNull();
    await fireEvent.click(getByRole("button", { name: "Stop all" }));

    const dialog = getByTestId("stop-all-dialog");
    expect(dialog.textContent).toContain("8 running agents");
    expect(dialog.textContent).toContain(
      "killed at the next iteration boundary",
    );
    expect(dialog.textContent).toContain("Branches, artifacts and memory");
    expect(dialog.textContent).toContain("Reversible for 10 minutes");

    await fireEvent.click(getByRole("button", { name: /Stop all — 8 agents/ }));
    expect(getByTestId("dispatch-stopped").textContent).toContain(
      "8 handoffs written, nothing lost",
    );
    expect(getByTestId("dispatch-stopped").textContent).toContain(
      "Restore previous state",
    );
  });

  it("renders schedules with cron and overlap-note from the live view", async () => {
    configureProjectsStub({
      runsPage: [],
    });
    const { getByTestId } = mount("schedules");

    expect(getByTestId("dispatch-schedules").textContent).toContain(
      "A cron expression fires a workflow",
    );
    expect(getByTestId("schedule-overlap-note").textContent).toContain(
      "Overlap is skipped, never queued",
    );
    // The stub serves 0 schedules: the empty state is honest (no schedules seeded).
    expect(
      getByTestId("schedule-cards").querySelectorAll("[data-schedule]").length,
    ).toBe(0);
    expect(getByTestId("schedule-executions").textContent).toBeTruthy();
  });

  it("renders every run with resolved models rather than tiers", async () => {
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
    const { getByTestId } = mount("runs");

    // The runs table is a live read now: wait for the page to land.
    await waitFor(() =>
      expect(getByTestId("dispatch-runs-table").textContent).toContain("run-1"),
    );

    const screen = getByTestId("dispatch-runs");
    expect(screen.textContent).toContain("Every run, scheduled or not");
    expect(
      [...getByTestId("dispatch-runs-table").querySelectorAll("th")].map(
        (header) => header.textContent,
      ),
    ).toEqual([
      "When",
      "Harness",
      "Project",
      "repo",
      "Agent",
      "role",
      "Model resolved",
      "Events",
      "Errors",
      "Tokens",
      "Verify",
      "Id",
    ]);
    expect(screen.textContent).toContain("unknown");
    expect(screen.textContent).not.toMatch(/\bxhigh\b|\bmedium\b/);
  });
});
