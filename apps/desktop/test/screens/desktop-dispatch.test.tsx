import { describe, expect, it } from "vitest";
import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { DispatchView } from "../../src/screens/dispatch/DispatchView";
import { configureProjectsStub } from "../projects/provider-stub";

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
import { DISPATCH_PROJECTS, SCHEDULES } from "../../src/fixtures/dispatch";

const mount = (tab: "autorun" | "schedules" | "runs" = "autorun") =>
  render(() => <DispatchView tab={tab} />);

describe("screens/desktop-dispatch", () => {
  it("renders autorun as a per-project switch with unavailable projects held off", () => {
    const { getByTestId } = mount();

    expect(getByTestId("dispatch-autorun").textContent).toContain(
      "Autorun is on or off, per project",
    );
    expect(
      getByTestId("autorun-projects").querySelectorAll("[data-project]").length,
    ).toBe(DISPATCH_PROJECTS.length);
    expect(
      getByTestId("autorun-project-weaver").getAttribute("data-state"),
    ).toBe("suspended");
    expect(getByTestId("autorun-project-amq").getAttribute("data-state")).toBe(
      "archived",
    );
    expect(
      (
        getByTestId("autorun-project-amq").querySelector(
          "button",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
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

  it("renders schedules with cron, skipped-overlap visibility, and recorded verify results", () => {
    const { getByTestId } = mount("schedules");

    expect(getByTestId("dispatch-schedules").textContent).toContain(
      "A cron expression fires a workflow",
    );
    expect(getByTestId("schedule-overlap-note").textContent).toContain(
      "Overlap is skipped, never queued",
    );
    expect(
      getByTestId("schedule-cards").querySelectorAll("[data-schedule]").length,
    ).toBe(SCHEDULES.length);
    expect(getByTestId("schedule-executions").textContent).toContain(
      "recorded with their verify result",
    );
    expect(getByTestId("schedule-executions").textContent).toContain(
      "previous execution still running",
    );
  });

  it("renders every run with resolved models rather than tiers", async () => {
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
