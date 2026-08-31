import {
  CONVERSATION,
  DRAFT_OUTPUTS,
  LIVE_LINE,
  PLANS,
  RECOMMENDATION,
  SCOPE_DECISION,
  SELECTED_PLAN_ID,
} from "../fixtures/plan";
import { isTauri } from "@tauri-apps/api/core";
import { streamFromCore } from "../transcript/from-core";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";
import type {
  DraftOutputs,
  PlanMessage,
  PlanSummary,
  PlanStep,
  Recommendation,
  ScopeDecision,
} from "../fixtures/plan";
import type { AgentEvent } from "../types/event";

export {
  ACP_LABEL,
  NEW_PLAN_NOTE,
  ONE_APPROVAL_RULE,
  PLAN_GRANULARITY_OPTIONS,
  PLAN_STEPS,
  PLAN_TASKS,
  SPEC_REQUIREMENTS,
} from "../fixtures/plan";
export type {
  DraftOutputs,
  PlanGranularity,
  PlanGranularityOption,
  PlanMessage,
  PlanState,
  PlanStep,
  PlanSummary,
  PlanTask,
  Recommendation,
  ScopeDecision,
  SpecOutput,
  SpecRequirement,
  Speaker,
} from "../fixtures/plan";

/** Live project-scoped plan list. */
export function fetchPlans(
  projectId: string,
): Promise<Envelope<PlanSummary[]>> {
  return dataProvider().query<PlanSummary>("plans_list", { projectId });
}

export interface PlanMutationReceipt {
  updated: boolean;
}

export interface PlanRequirementInput {
  id: string;
  body: string;
}

export function createPlan(
  projectId: string,
  title: string,
  goal: string,
): Promise<Envelope<PlanSummary>> {
  return dataProvider().queryOne<PlanSummary>("plan_create", {
    projectId,
    title,
    goal,
  });
}

export function setPlanStage(
  projectId: string,
  planId: string,
  stage: PlanStep,
  description = "",
): Promise<Envelope<PlanMutationReceipt>> {
  return dataProvider().queryOne<PlanMutationReceipt>("plan_stage_set", {
    projectId,
    planId,
    stage: stage.toLowerCase(),
    description,
  });
}

export function setPlanRequirements(
  projectId: string,
  planId: string,
  requirements: PlanRequirementInput[],
): Promise<Envelope<PlanMutationReceipt>> {
  return dataProvider().queryOne<PlanMutationReceipt>(
    "plan_requirements_set",
    { projectId, planId, requirements },
  );
}

/** Becomes: invoke("plans_list", { projectId }) — demo-only hook retained for component tests. */
export function usePlans(): PlanSummary[] {
  return PLANS;
}

/** Becomes: IPC-backed planning conversation. */
export function usePlanConversation(): PlanMessage[] {
  return CONVERSATION;
}

function planMessageFromEvent(event: AgentEvent): PlanMessage | null {
  if (!event.text || !["assistant", "thinking", "user"].includes(event.verb))
    return null;
  const speaker =
    event.verb === "user"
      ? "you"
      : event.verb === "thinking"
        ? "researcher"
        : "interviewer";
  return {
    id: `acp-${event.runId}-${event.seq}`,
    speaker,
    initials:
      speaker === "you" ? "YOU" : speaker === "researcher" ? "RE" : "IN",
    caption: speaker === "you" ? "you" : `${speaker} · ACP`,
    body: event.text,
    facts: [],
    finding: null,
  };
}

/** Subscribe the Plan conversation to the source-neutral IPC event channel. */
export async function subscribePlanConversationFromCore(
  onMessage: (message: PlanMessage) => void,
) {
  // No host (browser preview, tests) → resolve with nothing; the fixture stays on screen.
  if (!isTauri()) return;
  return streamFromCore((event) => {
    const message = planMessageFromEvent(event);
    if (message) onMessage(message);
  });
}

/** Becomes: invoke("plan_scope_decision", { planId }) */
export function usePlanScopeDecision(): ScopeDecision {
  return SCOPE_DECISION;
}

/** Becomes: invoke("plan_outputs", { planId }) */
export function usePlanOutputs(): DraftOutputs {
  return DRAFT_OUTPUTS;
}

/** Becomes: invoke("plan_recommendation", { planId }) */
export function usePlanRecommendation(): Recommendation {
  return RECOMMENDATION;
}

/** Becomes: the live line on the ACP stream. */
export function usePlanLiveLine(): string {
  return LIVE_LINE;
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultPlanId(): string {
  return SELECTED_PLAN_ID;
}
