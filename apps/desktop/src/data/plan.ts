import {
  CONVERSATION,
  DRAFT_OUTPUTS,
  LIVE_LINE,
  PLANS,
  RECOMMENDATION,
  SCOPE_DECISION,
  SELECTED_PLAN_ID,
} from '../fixtures/plan'
import { streamFromCore } from '../transcript/from-core'
import type { DraftOutputs, PlanMessage, PlanSummary, Recommendation, ScopeDecision } from '../fixtures/plan'
import type { AgentEvent } from '../types/event'

export { ACP_LABEL, NEW_PLAN_NOTE, ONE_APPROVAL_RULE, PLAN_STEPS } from '../fixtures/plan'
export type { DraftOutputs, PlanMessage, PlanState, PlanStep, PlanSummary, Recommendation, ScopeDecision, SpecOutput, Speaker } from '../fixtures/plan'

/** Becomes: invoke("plans_list") */
export function usePlans(): PlanSummary[] {
  return PLANS
}

/** Becomes: IPC-backed planning conversation. */
export function usePlanConversation(): PlanMessage[] {
  return CONVERSATION
}

function planMessageFromEvent(event: AgentEvent): PlanMessage | null {
  if (!event.text || !['assistant', 'thinking', 'user'].includes(event.verb)) return null
  const speaker = event.verb === 'user' ? 'you' : event.verb === 'thinking' ? 'researcher' : 'interviewer'
  return {
    id: `acp-${event.runId}-${event.seq}`,
    speaker,
    initials: speaker === 'you' ? 'YOU' : speaker === 'researcher' ? 'RE' : 'IN',
    caption: speaker === 'you' ? 'you' : `${speaker} · ACP`,
    body: event.text,
    facts: [],
    finding: null,
  }
}

/** Subscribe the Plan conversation to the source-neutral IPC event channel. */
export async function subscribePlanConversationFromCore(onMessage: (message: PlanMessage) => void) {
  return streamFromCore((event) => {
    const message = planMessageFromEvent(event)
    if (message) onMessage(message)
  })
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
