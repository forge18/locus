import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { INBOX_ITEMS } from "../../src/fixtures/inbox";
import { stopAllDispatch } from "../../src/data/dispatch";
import { DispatchPill } from "../../src/shell/DispatchPill";
import { InboxPill } from "../../src/shell/InboxPill";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const sessions = [
  {
    id: "run-1",
    label: "tapestry · builder@4",
    project: "tapestry",
    role: "builder",
    elapsed: "now",
    meta: "edit_file",
    needsAttention: true,
    lastActivityAt: 0,
  },
  {
    id: "run-2",
    label: "loom-db · builder@4",
    project: "loom-db",
    role: "builder",
    elapsed: "3m ago",
    meta: "running",
    needsAttention: false,
    lastActivityAt: 3,
  },
];

describe("M0.7 title-bar pills", () => {
  it("issues the supervisor stop command with handoff preservation", async () => {
    vi.mocked(invoke).mockResolvedValue({ snapshotId: "snapshot-1", stoppedRuns: 2 });

    await expect(stopAllDispatch()).resolves.toEqual({
      snapshotId: "snapshot-1",
      stoppedRuns: 2,
    });
    expect(invoke).toHaveBeenCalledWith("dispatch_stop_all", { writeHandoffs: true });
  });

  it("filters Dispatch activity and exposes stop-all/open actions", async () => {
    const onStopAll = vi.fn();
    const onOpenDispatch = vi.fn();
    const { getByTestId, getByRole } = render(() => (
      <DispatchPill
        running={2}
        needsYou={1}
        sessions={sessions}
        onStopAll={onStopAll}
        onOpenDispatch={onOpenDispatch}
      />
    ));

    await fireEvent.click(getByTestId("dispatch-pill"));
    expect(getByTestId("dispatch-activity-list").textContent).toContain(
      "tapestry",
    );
    expect(getByTestId("dispatch-activity-list").textContent).not.toContain(
      "loom-db",
    );

    await fireEvent.click(getByRole("button", { name: "All" }));
    expect(getByTestId("dispatch-activity-list").textContent).toContain(
      "loom-db",
    );
    await fireEvent.click(getByRole("button", { name: "Stop all" }));
    expect(onStopAll).toHaveBeenCalledOnce();

    await fireEvent.click(getByTestId("dispatch-pill"));
    await fireEvent.click(getByRole("button", { name: "Open Dispatch" }));
    expect(onOpenDispatch).toHaveBeenCalledOnce();
  });

  it("shows Inbox response rows before opening the full Inbox", async () => {
    const onOpenInbox = vi.fn();
    const { getByTestId, getByRole } = render(() => (
      <InboxPill
        count={INBOX_ITEMS.length}
        items={INBOX_ITEMS}
        onOpenInbox={onOpenInbox}
      />
    ));

    await fireEvent.click(getByTestId("inbox-pill"));
    expect(getByTestId("inbox-preview-items").textContent).toContain(
      INBOX_ITEMS[0].title,
    );
    await fireEvent.click(getByRole("button", { name: "Open Inbox" }));
    expect(onOpenInbox).toHaveBeenCalledOnce();
  });
});
