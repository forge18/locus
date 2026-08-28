export interface NormalizedWorkflowEvent {
  nodeId: string;
  state: string;
  iteration: number;
  tokens: number;
  wallClockMs: number;
}

/** One normalized event stream feeds the canvas overlay and transcript projections. */
export const WORKFLOW_EVENTS: NormalizedWorkflowEvent[] = [
  {
    nodeId: "n-plan",
    state: "done",
    iteration: 1,
    tokens: 1840,
    wallClockMs: 8200,
  },
  {
    nodeId: "n-gate",
    state: "passed",
    iteration: 1,
    tokens: 210,
    wallClockMs: 900,
  },
  {
    nodeId: "n-build",
    state: "running",
    iteration: 2,
    tokens: 12600,
    wallClockMs: 42100,
  },
  {
    nodeId: "n-verify",
    state: "failed 1/3",
    iteration: 2,
    tokens: 640,
    wallClockMs: 3100,
  },
  {
    nodeId: "n-cond",
    state: "evaluable",
    iteration: 2,
    tokens: 12,
    wallClockMs: 2,
  },
  {
    nodeId: "n-review",
    state: "idle",
    iteration: 2,
    tokens: 0,
    wallClockMs: 0,
  },
];

export function workflowEventsForTranscript(): NormalizedWorkflowEvent[] {
  return WORKFLOW_EVENTS;
}
