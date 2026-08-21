import {
  CONVERSATION,
  DRAFT_OUTPUTS,
  LIVE_LINE,
  PLANS,
  RECOMMENDATION,
  SCOPE_DECISION,
  SELECTED_PLAN_ID,
} from '../fixtures/plan'
import type { DraftOutputs, PlanMessage, PlanSummary, Recommendation, ScopeDecision } from '../fixtures/plan'

export { ACP_LABEL, NEW_PLAN_NOTE, ONE_APPROVAL_RULE, PLAN_STEPS } from '../fixtures/plan'
export type { DraftOutputs, PlanMessage, PlanState, PlanStep, PlanSummary, Recommendation, ScopeDecision, SpecOutput, Speaker } from '../fixtures/plan'

/** Becomes: invoke("plans_list") */
export function usePlans(): PlanSummary[] {
  return PLANS
}

/** Becomes: Channel<AgentEvent>("acp_session_update") */
export function usePlanConversation(): PlanMessage[] {
  return CONVERSATION
}

/** Becomes: invoke("plan_scope_decision", { planId }) */
export function usePlanScopeDecision(): ScopeDecision {
  return SCOPE_DECISION
}

/** Becomes: invoke("plan_outputs", { planId }) */
export function usePlanOutputs(): DraftOutputs {
  return DRAFT_OUTPUTS
}

/** Becomes: invoke("plan_recommendation", { planId }) */
export function usePlanRecommendation(): Recommendation {
  return RECOMMENDATION
}

/** Becomes: the live line on the ACP stream. */
export function usePlanLiveLine(): string {
  return LIVE_LINE
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultPlanId(): string {
  return SELECTED_PLAN_ID
}
