import { Show } from 'solid-js'
import type { Metric } from '../../data/status'

export interface MetricCardProps {
  metric: Metric
}

/**
 * A null value renders *unknown*. Zero is a measurement; unknown is the absence of
 * one, and a harness that reports no usage has given us the second.
 */
export function MetricCard(props: MetricCardProps) {
  return (
    <div
      class={['metric-card', props.metric.attention ? 'metric-card-attention' : '']
        .filter(Boolean)
        .join(' ')}
      data-testid={`metric-${props.metric.label.toLowerCase().replace(/\s+/g, '-')}`}
      data-attention={props.metric.attention ? 'true' : undefined}
    >
      <span class="metric-label">{props.metric.label}</span>
      <div class="metric-value">
        <Show
          when={props.metric.value !== null}
          fallback={
            <span class="metric-unknown" data-testid="metric-unknown">
              unknown
            </span>
          }
        >
          <span class="metric-numeral" data-testid="metric-numeral">
            {props.metric.value}
          </span>
          <Show when={props.metric.unit}>
            <span class="metric-unit" data-testid="metric-unit">
              {props.metric.unit}
            </span>
          </Show>
        </Show>
      </div>
      <Show when={props.metric.note || props.metric.badNote}>
        <span
          class={['metric-note', props.metric.badNote ? 'metric-note-bad' : '']
            .filter(Boolean)
            .join(' ')}
          data-testid="metric-note"
        >
          {props.metric.badNote ?? props.metric.note}
        </span>
      </Show>
    </div>
  )
}
