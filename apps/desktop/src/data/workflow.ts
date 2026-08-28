import {
  ARROW_MARKERS,
  CANVAS_EDGES,
  CANVAS_NODES,
  EXPRESSION,
  GUARDRAILS,
  GUARDRAIL_TRIPS,
  LOOP_GROUP,
  OPERANDS,
  PALETTE,
  PRESETS,
  WORKFLOW,
} from "../fixtures/workflow";
import type {
  CanvasEdge,
  CanvasNode,
  Clause,
  Guardrail,
  PaletteNode,
  Preset,
} from "../fixtures/workflow";
import {
  WORKFLOW_EVENTS,
  workflowEventsForTranscript,
} from "./workflow-events";
import type { NormalizedWorkflowEvent } from "./workflow-events";
import type { GuardrailTrip, WorkflowDef } from "../types/workflows";

export {
  ARROW_MARKERS,
  BUDGET_NOTE,
  COMPILED,
  COMPILED_NOTE,
  LOOP_GROUP,
  NO_MODEL_NOTE,
  OPERAND_NOTE,
  PRESET_NOTE,
  WAITING_NOTE,
  ZOOM,
} from "../fixtures/workflow";
export type {
  CanvasEdge,
  CanvasNode,
  Clause,
  Guardrail,
  PaletteNode,
  Preset,
} from "../fixtures/workflow";
export type { NormalizedWorkflowEvent } from "./workflow-events";

/** Becomes: invoke("workflow_node_vocabulary") */
export function usePalette(): PaletteNode[] {
  return PALETTE;
}

/** Becomes: invoke("workflow_presets") */
export function usePresets(): Preset[] {
  return PRESETS;
}

/** Becomes: invoke("workflow_def", { id }) */
export function useWorkflow(): WorkflowDef {
  return WORKFLOW;
}

/** Becomes: invoke("workflow_graph", { id }) */
export function useCanvas(): {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  loop: typeof LOOP_GROUP;
  markers: readonly string[];
  events: NormalizedWorkflowEvent[];
} {
  return {
    nodes: CANVAS_NODES,
    edges: CANVAS_EDGES,
    loop: LOOP_GROUP,
    markers: ARROW_MARKERS,
    events: WORKFLOW_EVENTS,
  };
}

/** Becomes: invoke("condition_expression", { nodeId }) */
export function useExpression(): Clause[] {
  return EXPRESSION;
}

/** Becomes: invoke("condition_operands") — every one is a column. */
export function useOperands(): readonly string[] {
  return OPERANDS;
}

/** Becomes: invoke("workflow_guardrails", { id }) */
export function useGuardrails(): Guardrail[] {
  return GUARDRAILS;
}

/** Becomes: subscribe("workflow.events", { id }) */
export function useWorkflowEvents(): NormalizedWorkflowEvent[] {
  return workflowEventsForTranscript();
}

/** Becomes: emit("guardrail_tripped") */
export function useGuardrailTrips(): GuardrailTrip[] {
  return GUARDRAIL_TRIPS;
}
