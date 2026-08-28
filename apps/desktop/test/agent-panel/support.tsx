import { render } from "@solidjs/testing-library";
import type { JSX } from "solid-js";
import { AgentPane } from "../../src/panes/AgentPane";
import type {
  AgentPaneCheckpoint,
  AgentPaneElicitation,
  AgentPaneSession,
  AgentPanePlan,
} from "../../src/panes/agent-panel-model";
import type { AgentEvent, EventVerb } from "../../src/types/event";

export const session: AgentPaneSession = {
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
  status: "working",
};

export function event(
  verb: EventVerb,
  seq = 0,
  extra: Partial<AgentEvent> = {},
): AgentEvent {
  return {
    id: `support-event-${seq}`,
    runId: "run-1",
    seq,
    ts: "now",
    verb,
    raw: {},
    ...extra,
  };
}

export const plan: AgentPanePlan = {
  id: "plan-1",
  title: "Build and verify",
  steps: [
    { id: "read", title: "Read the channel", status: "done" },
    { id: "wire", title: "Wire the stream", status: "in_progress" },
  ],
};

export const checkpoints: AgentPaneCheckpoint[] = [
  { id: "checkpoint-1", label: "Before channel edit", file: "src/channel.ts", state: "available" },
];

export const elicitation: AgentPaneElicitation = {
  id: "ask-1",
  title: "Choose a transport",
  detail: "Confirm the transport before the next turn.",
  fields: [{ id: "transport", label: "Transport", type: "text", required: true }],
};

export function mount(children: JSX.Element = <AgentPane runId="run-1" live={false} session={session} />) {
  return render(() => children);
}
