// schema: workflows.workflow_defs + workflows.executions + workflows.guardrail_trips
// replaced by: invoke("workflow_def") + emit("iteration_finished")

import type { GuardrailTrip, WorkflowDef } from "../types/workflows";

/** The node vocabulary. Verify is required in every graph, which is why it is marked. */
export interface PaletteNode {
  kind: string;
  label: string;
  icon: string;
  /** native | condition — decides the tint. */
  tone: "default" | "condition";
  required: boolean;
}

export const PALETTE: PaletteNode[] = [
  {
    kind: "agent",
    label: "Agent",
    icon: "robot",
    tone: "default",
    required: false,
  },
  {
    kind: "task",
    label: "Task",
    icon: "check-square",
    tone: "default",
    required: false,
  },
  {
    kind: "loop",
    label: "Loop",
    icon: "infinity",
    tone: "default",
    required: false,
  },
  {
    kind: "condition",
    label: "Condition",
    icon: "arrows-split",
    tone: "condition",
    required: false,
  },
  {
    kind: "gate",
    label: "Gate",
    icon: "hand-palm",
    tone: "default",
    required: false,
  },
  {
    kind: "verify",
    label: "Verify",
    icon: "flag-checkered",
    tone: "default",
    required: true,
  },
];

export interface Preset {
  name: string;
  note: string;
}

export const PRESETS: Preset[] = [
  { name: "Ralph loop", note: "pick · act · validate · commit · reset" },
  { name: "Review pass", note: "read-only tools, one reviewer, one gate" },
];

export const PRESET_NOTE =
  "A preset expands into ordinary nodes, so it can be edited rather than configured.";

export const NO_MODEL_NOTE =
  "No model in the orchestration path — the graph decides";

export const ZOOM = "100%";

export interface CanvasNode {
  id: string;
  kind: string;
  label: string;
  state: string;
  x: number;
  y: number;
  tone: "default" | "condition";
}

export const CANVAS_NODES: CanvasNode[] = [
  {
    id: "n-plan",
    kind: "agent",
    label: "planner@3",
    state: "done",
    x: 40,
    y: 150,
    tone: "default",
  },
  {
    id: "n-gate",
    kind: "gate",
    label: "Approve plan",
    state: "passed",
    x: 250,
    y: 150,
    tone: "default",
  },
  {
    id: "n-build",
    kind: "agent",
    label: "builder@4",
    state: "running",
    x: 460,
    y: 100,
    tone: "default",
  },
  {
    id: "n-verify",
    kind: "verify",
    label: "cargo test",
    state: "failed 1/3",
    x: 460,
    y: 230,
    tone: "default",
  },
  {
    id: "n-cond",
    kind: "condition",
    label: "verify.passed",
    state: "evaluable",
    x: 670,
    y: 165,
    tone: "condition",
  },
  {
    id: "n-review",
    kind: "agent",
    label: "reviewer@2",
    state: "idle",
    x: 700,
    y: 40,
    tone: "default",
  },
];

export interface CanvasEdge {
  from: string;
  to: string;
  label: string | null;
  /** A loop-back edge is dashed, because it is not forward progress. */
  dashed: boolean;
}

export const CANVAS_EDGES: CanvasEdge[] = [
  { from: "n-plan", to: "n-gate", label: null, dashed: false },
  { from: "n-gate", to: "n-build", label: "approved", dashed: false },
  { from: "n-build", to: "n-verify", label: null, dashed: false },
  { from: "n-verify", to: "n-cond", label: null, dashed: false },
  { from: "n-cond", to: "n-build", label: "failed · retry", dashed: true },
  { from: "n-cond", to: "n-review", label: "passed", dashed: false },
];

/** The dashed rounded rect that groups the retry loop. */
export const LOOP_GROUP = {
  x: 440,
  y: 80,
  width: 400,
  height: 210,
  label: "loop · max 3",
};

/** The four arrow markers the edge layer defines: one per edge tone. */
export const ARROW_MARKERS = [
  "arrow-default",
  "arrow-pass",
  "arrow-fail",
  "arrow-loop",
];

export interface Clause {
  operand: string;
  operator: string;
  value: string;
}

export const EXPRESSION: Clause[] = [
  { operand: "verify.passed", operator: "==", value: "true" },
  { operand: "iteration", operator: "<", value: "3" },
];

export const COMPILED = "verify.passed == true and iteration < 3";

export const COMPILED_NOTE =
  "total · evaluable in the core · reproducible from stored events";

/**
 * Every operand is a column. There is no code, no model and no I/O in a Condition
 * — which is exactly why anything it cannot express has to be a Gate.
 */
export const OPERANDS = [
  "verify.passed",
  "verify.exit_code",
  "iteration",
  "elapsed",
  "tokens.used",
  "events.count(tool_error)",
  "events.last(kind)",
  "artifact.exists(kind)",
  "task.status",
  "mail.pending",
];

export const OPERAND_NOTE =
  "No code, no model, no I/O — anything this cannot express is a Gate.";

export interface Guardrail {
  key: string;
  label: string;
  kind: "stepper" | "toggle" | "value";
  value: string;
}

export const GUARDRAILS: Guardrail[] = [
  {
    key: "max_iterations",
    label: "max_iterations",
    kind: "stepper",
    value: "8",
  },
  {
    key: "reflection_before_retry",
    label: "reflection before retry",
    kind: "toggle",
    value: "on",
  },
  {
    key: "kill_and_reassign",
    label: "kill & reassign after",
    kind: "stepper",
    value: "3",
  },
  {
    key: "idle_detection",
    label: "idle detection",
    kind: "value",
    value: "60s",
  },
  { key: "wall_clock", label: "wall clock", kind: "value", value: "none" },
  { key: "token_budget", label: "token budget", kind: "value", value: "none" },
];

export const BUDGET_NOTE =
  "An unset budget is unbounded on purpose: a budget you have not measured stops good runs before it stops bad ones.";

export const WAITING_NOTE = "Waiting ≠ idle";

export const WORKFLOW: WorkflowDef = {
  id: "wf-1",
  name: "build-and-verify",
  version: 4,
  nodes: CANVAS_NODES.map((n) => ({
    id: n.id,
    kind: n.kind === "task" || n.kind === "loop" ? "agent" : (n.kind as never),
    label: n.label,
    agent: n.kind === "agent" ? n.label : null,
    verifyCommand: n.kind === "verify" ? "cargo test -p tapestry-core" : null,
    x: n.x,
    y: n.y,
  })),
  edges: CANVAS_EDGES.map((e) => ({ from: e.from, to: e.to, label: e.label })),
  spec: "One plan, one gate, then per-task build-verify with at most three iterations before the guardrail reassigns.",
  updatedAt: "2026-08-20T09:12:00Z",
};

export const GUARDRAIL_TRIPS: GuardrailTrip[] = [
  {
    id: "gt-1",
    executionId: "ex-1",
    kind: "stuck",
    action: "kill_and_reassign",
    at: "2026-08-20T14:28:00Z",
    detail: "three iterations on cargo test -p weaver parser::",
  },
  {
    id: "gt-2",
    executionId: "ex-1",
    kind: "budget",
    action: "warn",
    at: "2026-08-20T12:04:00Z",
    detail: "102.3k tokens on one task",
  },
];
