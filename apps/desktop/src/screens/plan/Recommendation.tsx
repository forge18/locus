import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import type { Recommendation as RecommendationData } from '../../data/plan'

export interface RecommendationProps {
  recommendation: RecommendationData
  onApprove: () => void
}

/**
 * Confidence is a named condition, not a number alone. "0.62" is not an action;
 * "high once the provenance tie-break is settled" is, so the card carries both and
 * the condition is not optional.
 */
export function Recommendation(props: RecommendationProps) {
  return (
    <section class="output-card recommendation" data-testid="recommendation">
      <div class="output-card-head">
        <Icon name="seal-check" size={12} style={{ color: 'var(--ac)' }} />
        recommendation
      </div>

      <div class="recommendation-figure">
        <span class="recommendation-confidence" data-testid="recommendation-confidence">
          {props.recommendation.confidence.toFixed(2)}
        </span>
        <span class="recommendation-label">confidence</span>
      </div>

      <span class="recommendation-condition" data-testid="recommendation-condition">
        {props.recommendation.condition}
      </span>

      <span class="recommendation-open" data-testid="recommendation-open">
        open[{props.recommendation.open}]
      </span>

      <span class="output-line" data-testid="recommendation-ratchet">
        {props.recommendation.ratchet}
      </span>

      <Button variant="primary" block data-testid="recommendation-approve" onClick={props.onApprove}>
        Approve — {props.recommendation.taskCount} tasks to the board
      </Button>
    </section>
  )
}
