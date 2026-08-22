import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import type { ScopeDecision as ScopeDecisionData } from '../../data/plan'

export interface ScopeDecisionProps {
  decision: ScopeDecisionData
  onWiden: () => void
  onKeepOut: () => void
}

/**
 * Renders in the message flow, in sequence, with no overlay and no portal. A scope
 * change is a decision the conversation makes on the spot — PLAN.md counts it apart
 * from a question for exactly that reason, and turning it into a gate would send it
 * to the inbox, which is not where it goes.
 */
export function ScopeDecision(props: ScopeDecisionProps) {
  return (
    <div class="scope-decision" data-testid="scope-decision">
      <Icon name="arrows-split" size={13} style={{ color: 'var(--action-attention)', 'flex-shrink': 0 }} />
      <div>
        <div class="scope-decision-title" data-testid="scope-decision-title">
          {props.decision.question}
        </div>
        <div class="scope-decision-detail" innerHTML={mono(props.decision.detail)} />
        <div class="scope-decision-actions">
          <Button variant="primary" data-testid="scope-widen" onClick={props.onWiden}>
            {props.decision.widen}
          </Button>
          <Button variant="secondary" data-testid="scope-keep-out" onClick={props.onKeepOut}>
            {props.decision.keepOut}
          </Button>
        </div>
      </div>
    </div>
  )
}

/** Backticked spans become mono. The source is fixture prose we author. */
function mono(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/`([^`]+)`/g, '<code class="mono">$1</code>')
}
