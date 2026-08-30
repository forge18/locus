import { WORKFLOW_EVENTS } from "../fixtures/workflow-events";
import type { NormalizedWorkflowEvent } from "../fixtures/workflow-events";

export { WORKFLOW_EVENTS };
export type { NormalizedWorkflowEvent };

/** Becomes: subscribe("workflow.events", { id }) */
export function workflowEventsForTranscript(): NormalizedWorkflowEvent[] {
  return WORKFLOW_EVENTS;
}
