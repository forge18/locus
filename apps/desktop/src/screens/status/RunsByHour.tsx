import { For } from 'solid-js'
import type { HourBar } from '../../data/status'

export interface RunsByHourProps {
  hours: HourBar[]
}

/** Accent passed, --bad failed, --blue-lit aborted, stacked bottom-up. */
const SEGMENTS = [
  { key: 'passed', class: 'hour-seg-passed' },
  { key: 'failed', class: 'hour-seg-failed' },
  { key: 'aborted', class: 'hour-seg-aborted' },
] as const

export function RunsByHour(props: RunsByHourProps) {
  const max = () =>
    Math.max(...props.hours.map((h) => h.passed + h.failed + h.aborted), 1)

  return (
    <section class="panel" data-testid="runs-by-hour">
      <div class="panel-head">
        <span class="panel-title">Runs by hour</span>
        <div class="panel-legend" data-testid="hours-legend">
          <span>
            <i class="hour-seg-passed" />
            passed
          </span>
          <span>
            <i class="hour-seg-failed" />
            failed
          </span>
          <span>
            <i class="hour-seg-aborted" />
            aborted
          </span>
        </div>
      </div>
      <div class="hours" data-testid="hours">
        <For each={props.hours}>
          {(hour) => {
            const total = hour.passed + hour.failed + hour.aborted
            return (
              <div
                class="hour-bar"
                data-testid={`hour-${hour.hour}`}
                style={{ height: `${(total / max()) * 100}%` }}
              >
                {/* column-reverse, so the first segment sits at the bottom */}
                <For each={SEGMENTS}>
                  {(seg) => (
                    <div
                      class={seg.class}
                      data-segment={seg.key}
                      style={{ height: total ? `${(hour[seg.key] / total) * 100}%` : '0%' }}
                    />
                  )}
                </For>
              </div>
            )
          }}
        </For>
      </div>
      <div class="hour-axis" data-testid="hour-axis">
        <For each={props.hours}>
          {(hour, i) => <span>{i() % 3 === 0 ? hour.hour : ''}</span>}
        </For>
      </div>
    </section>
  )
}
