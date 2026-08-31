import type { NormalizedWorkflowEvent } from "./demo/fixtures/workflow-events";
import { dataProvider } from "./provider";

export { WORKFLOW_EVENTS } from "./demo/fixtures/workflow-events";
export type { NormalizedWorkflowEvent };

/** Becomes: subscribe("workflow.events", { id }) */
export function workflowEventsForTranscript(): NormalizedWorkflowEvent[] {
  return (
    dataProvider().read?.<NormalizedWorkflowEvent[]>("workflow_events") ?? []
  );
}
