import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import type {
  AgentPaneCheckpoint,
  AgentPaneElicitation,
  AgentPaneFinding,
  AgentPanePlan,
  AgentPaneSession,
} from "../../src/panes/agent-panel-model";
import type { AgentEvent } from "../../src/types/event";

const session = (
  status: AgentPaneSession["status"] = "idle",
): AgentPaneSession => ({
  project: "p-tapestry",
  task: "task-42",
  workflow: "workflow-1",
  agent: "builder@4",
  model: "model-1",
  harness: "pi",
  effort: "high",
  name: "Thread the channel",
  context: { used: 12_000, total: 200_000 },
  cost: "$0.42",
  permissionPosture: "gated",
  status,
});

function event(
  verb: AgentEvent["verb"],
  seq: number,
  extra: Partial<AgentEvent> = {},
): AgentEvent {
  return {
    id: `event-${seq}`,
    runId: "run-1",
    seq,
    ts: "now",
    verb,
    raw: {},
    ...extra,
  };
}

const plan: AgentPanePlan = {
  id: "plan-1",
  title: "Build and verify",
  steps: [
    { id: "step-1", title: "Read the channel", status: "done" },
    { id: "step-2", title: "Wire the stream", status: "in_progress" },
  ],
};

const findings: AgentPaneFinding[] = [
  {
    id: "finding-1",
    title: "ACP owns the stream",
    summary: "The panel should render normalized events rather than a PTY.",
    source: "docs/acp.md",
    provenance: "seed",
  },
];

const checkpoints: AgentPaneCheckpoint[] = [
  {
    id: "checkpoint-1",
    label: "Before channel edit",
    file: "src/channel.ts",
    state: "available",
  },
];

const elicitation: AgentPaneElicitation = {
  id: "ask-1",
  title: "Choose a transport",
  detail: "Confirm the transport before the next turn.",
  fields: [
    { id: "transport", label: "Transport", type: "text", required: true },
  ],
};

describe("agent-panel", () => {
  it("mounts the flexible no-PTY layout and toggles context, cost, research, and menu", async () => {
    const { getByTestId, getByRole, getByText } = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session()}
        findings={findings}
        events={[event("assistant", 0, { text: "Done" })]}
      />
    ));
    expect(getByTestId("agent-pane").getAttribute("data-pty")).toBe("false");
    expect(getByTestId("agent-panel-header")).toBeTruthy();
    expect(getByTestId("agent-stream")).toBeTruthy();
    expect(getByTestId("agent-composer")).toBeTruthy();
    await fireEvent.click(getByTestId("agent-context-toggle"));
    expect(getByTestId("agent-context-view")).toBeTruthy();
    await fireEvent.click(getByTestId("agent-cost-toggle"));
    expect(getByTestId("agent-cost-toggle").getAttribute("aria-pressed")).toBe(
      "true",
    );
    await fireEvent.click(getByRole("button", { name: "Auto" }));
    expect(getByRole("button", { name: "Auto" }).getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(getByTestId("agent-research-toggle"));
    expect(getByTestId("agent-research-pane")).toBeTruthy();
    await fireEvent.click(getByTestId("agent-overflow-toggle"));
    expect(getByRole("menu")).toBeTruthy();
    expect(getByText("Clear context")).toBeTruthy();
  });

  it("keeps ACP event families visually distinct and supports disclosures, gates, plans, files, and restore", async () => {
    const onApprove = vi.fn();
    const onDecline = vi.fn();
    const onOpenFile = vi.fn();
    const onRestore = vi.fn();
    const events = [
      event("user", 0, { text: "Please inspect the channel." }),
      event("assistant", 1, {
        text: "I found the channel.",
        raw: { path: "src/channel.ts" },
      }),
      event("thinking", 2, { text: "First sentence. More detail follows." }),
      event("tool_call", 3, {
        tool: "read_file",
        args: { path: "src/channel.ts" },
      }),
      event("tool_error", 4, { tool: "run_tests", text: "test failed" }),
      event("permission_request", 5, {
        raw: { diff: { path: "src/channel.ts", before: "old", after: "new" } },
      }),
    ];
    const { getByText, getByTestId, getAllByTestId, getByRole } = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session()}
        events={events}
        plan={plan}
        checkpoints={checkpoints}
        onApprovePermission={onApprove}
        onDeclinePermission={onDecline}
        onOpenFile={onOpenFile}
        onRestoreCheckpoint={onRestore}
      />
    ));
    expect(getByTestId("agent-user-card")).toBeTruthy();
    expect(getByTestId("agent-turn")).toBeTruthy();
    expect(getByTestId("agent-thinking-block")).toBeTruthy();
    expect(
      getAllByTestId("agent-tool-card").find(
        (card) => card.getAttribute("data-tool-status") === "running",
      ),
    ).toBeTruthy();
    expect(getByTestId("agent-permission-card")).toBeTruthy();
    expect(getByTestId("agent-inline-diff")).toBeTruthy();
    await fireEvent.click(getByText("Decline"));
    expect(onDecline).toHaveBeenCalledTimes(1);
    await fireEvent.click(getByText("Expand"));
    await fireEvent.click(getByText("Full"));
    expect(getByTestId("agent-thinking-block").textContent).toContain(
      "More detail follows",
    );
    await fireEvent.click(getByText("Build and verify"));
    expect(getByTestId("agent-plan-dock").textContent).toContain(
      "Wire the stream",
    );
    await fireEvent.click(
      getByTestId("agent-checkpoints").querySelector("[data-file-path]")!,
    );
    expect(onOpenFile).toHaveBeenCalledWith("src/channel.ts");
    await fireEvent.click(getByRole("button", { name: "Restore" }));
    expect(onRestore).toHaveBeenCalledWith(checkpoints[0]);
  });

  it("validates elicitation before accepting and exposes slash commands and stop", async () => {
    const onAccept = vi.fn();
    const onStop = vi.fn();
    const { getByRole, getByLabelText, getByTestId } = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session("waiting")}
        elicitation={elicitation}
        onAcceptElicitation={onAccept}
      />
    ));
    await fireEvent.click(getByRole("button", { name: "Accept" }));
    expect(getByRole("alert").textContent).toContain("Transport is required");
    await fireEvent.input(getByLabelText("Transport"), {
      target: { value: "ACP" },
    });
    await fireEvent.click(getByRole("button", { name: "Accept" }));
    expect(onAccept).toHaveBeenCalledWith(elicitation, { transport: "ACP" });
    await fireEvent.input(getByLabelText("Message agent"), {
      target: { value: "/" },
    });
    expect(getByTestId("agent-composer").textContent).toContain("/new");
    const runningPane = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session("working")}
        onStop={onStop}
      />
    ));
    await fireEvent.click(runningPane.getByRole("button", { name: "Stop" }));
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("reviews and promotes research only after the close review", async () => {
    const onReview = vi.fn();
    const onPromote = vi.fn();
    const { getByTestId, getByRole } = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session()}
        findings={findings}
        onReviewFinding={onReview}
        onPromoteFinding={onPromote}
      />
    ));
    await fireEvent.click(getByTestId("agent-research-toggle"));
    await fireEvent.click(getByRole("button", { name: "Review for close" }));
    expect(onReview).toHaveBeenCalledWith(findings[0]);
    await fireEvent.click(getByRole("button", { name: "Promote at close" }));
    expect(onPromote).toHaveBeenCalledWith(findings[0]);
  });

  it("renders rich output, line-oriented gated diffs, and pins citations into research", async () => {
    const onPin = vi.fn();
    const onRemaining = vi.fn();
    const citation = {
      id: "citation-1",
      label: "ACP contract",
      source: "docs/acp.md",
      summary: "The panel consumes normalized events.",
    };
    const { getByTestId, getByRole, getByText, getAllByTestId } = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session("working")}
        events={[
          event("assistant", 0, {
            text: "Answer.\n\n```ts\nconst value = 1;\n```\n\n| kind | state |\n| --- | --- |\n| ACP | live |\n\n![diagram](https://example.com/diagram.png)",
            raw: { citation },
          }),
          event("permission_request", 1, {
            raw: {
              diff: {
                path: "src/channel.ts",
                before: "old\nsame",
                after: "new\nsame",
              },
            },
          }),
        ]}
        onPinCitation={onPin}
        onApproveRemainingTurn={onRemaining}
      />
    ));
    expect(getByTestId("agent-turn").querySelector("pre")).toBeTruthy();
    expect(getByTestId("agent-turn").querySelector("table")).toBeTruthy();
    expect(getByTestId("agent-turn").querySelector("img")).toBeTruthy();
    expect(getAllByTestId("agent-diff-row")).toHaveLength(3);
    await fireEvent.click(getByTestId("agent-pin-citation"));
    expect(onPin).toHaveBeenCalledWith(citation);
    await fireEvent.click(
      getByRole("button", { name: "Approve remaining this turn" }),
    );
    expect(onRemaining).toHaveBeenCalledTimes(1);
    expect(getByTestId("agent-live-status").textContent).toContain("working");
    await fireEvent.click(getByTestId("agent-research-toggle"));
    expect(getByText("ACP contract")).toBeTruthy();
  });

  it("queues steering while running and dispatches session slash commands", async () => {
    const onQueue = vi.fn();
    const onStop = vi.fn();
    const onClear = vi.fn();
    const runningPane = render(() => (
      <AgentPane
        runId="run-1"
        live={false}
        session={session("working")}
        onQueue={onQueue}
        onStop={onStop}
      />
    ));
    await fireEvent.input(runningPane.getByLabelText("Message agent"), {
      target: { value: "check the next turn" },
    });
    await fireEvent.submit(runningPane.getByTestId("agent-composer"));
    expect(onQueue).toHaveBeenCalledWith("check the next turn");
    expect(
      runningPane.getByTestId("agent-queued-prompts").textContent,
    ).toContain("check the next turn");
    await fireEvent.click(runningPane.getByRole("button", { name: "Stop" }));
    expect(onStop).toHaveBeenCalledTimes(1);

    const commandPane = render(() => (
      <AgentPane runId="run-2" live={false} onClearContext={onClear} />
    ));
    await fireEvent.input(commandPane.getByLabelText("Message agent"), {
      target: { value: "/clear" },
    });
    await fireEvent.submit(commandPane.getByTestId("agent-composer"));
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it("retains elicitation values after decline and offers history suggestions", async () => {
    const request: AgentPaneElicitation = {
      ...elicitation,
      fields: [{ ...elicitation.fields[0], suggestions: ["HTTP"] }],
      history: [{ transport: "ACP" }],
    };
    const view = render(() => (
      <AgentPane runId="run-1" live={false} elicitation={request} />
    ));
    await fireEvent.input(view.getByLabelText("Transport"), {
      target: { value: "HTTP" },
    });
    expect(
      view.getByTestId("agent-elicitation").querySelector("datalist"),
    ).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Decline" }));
    expect((view.getByLabelText("Transport") as HTMLInputElement).value).toBe(
      "HTTP",
    );
  });
});
