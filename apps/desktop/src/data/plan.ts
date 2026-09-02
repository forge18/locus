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
} from "./demo/fixtures/plan";
import type { AgentEvent } from "../types/event";

export {
  ACP_LABEL,
  NEW_PLAN_NOTE,
  ONE_APPROVAL_RULE,
  PLAN_GRANULARITY_OPTIONS,
  PLAN_STEPS,
  PLAN_TASKS,
  SPEC_REQUIREMENTS,
} from "./demo/fixtures/plan";
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
} from "./demo/fixtures/plan";

/** Live project-scoped plan list. */
export function fetchPlans(
  projectId?: string,
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
  return dataProvider().queryOne<PlanMutationReceipt>("plan_requirements_set", {
    projectId,
    planId,
    requirements,
  });
}

/** Becomes: invoke("plans_list", { projectId }) — demo-only hook retained for component tests. */
export function usePlans(): PlanSummary[] {
  return dataProvider().read?.<PlanSummary[]>("plans_list") ?? [];
}

/** Becomes: IPC-backed planning conversation. */
export function usePlanConversation(): PlanMessage[] {
  return dataProvider().read?.<PlanMessage[]>("plan_conversation") ?? [];
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
  return (
    dataProvider().read?.<ScopeDecision>("plan_scope_decision") ?? {
      question: "Scope decision unavailable",
      detail: "The planning backend has not provided a scope decision.",
      widen: "Unavailable",
      keepOut: "Unavailable",
    }
  );
}

/** Becomes: invoke("plan_outputs", { planId }) */
export function usePlanOutputs(): DraftOutputs {
  return (
    dataProvider().read?.<DraftOutputs>("plan_outputs") ?? {
      spec: { name: "spec.md", lines: [] },
      tasks: [],
      tools: [],
      newTools: [],
    }
  );
}

/** Becomes: invoke("plan_recommendation", { planId }) */
export function usePlanRecommendation(): Recommendation {
  return (
    dataProvider().read?.<Recommendation>("plan_recommendation") ?? {
      confidence: 0,
      open: 0,
      condition: "Recommendation unavailable",
      ratchet: "The planning backend has not provided a recommendation.",
      taskCount: 0,
    }
  );
}

/** Becomes: the live line on the ACP stream. */
export function usePlanLiveLine(): string {
  return dataProvider().read?.<string>("plan_live_line") ?? "";
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultPlanId(): string {
  return dataProvider().read?.<string>("plan_default_id") ?? "";
}
