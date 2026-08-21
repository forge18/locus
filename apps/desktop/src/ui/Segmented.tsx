import { For } from 'solid-js'
import { SegmentedControl } from '@kobalte/core/segmented-control'

export interface SegmentedOption {
  value: string
  label: string
}

export interface SegmentedProps {
  options: SegmentedOption[]
  value: string
  onChange: (value: string) => void
  /** Reader-facing name for the group. */
  label: string
}

/** The active segment is accent — line and text, not a fill. */
export function Segmented(props: SegmentedProps) {
  return (
    <SegmentedControl
      class="seg"
      value={props.value}
      onChange={props.onChange}
      aria-label={props.label}
    >
      <For each={props.options}>
        {(o) => (
          <SegmentedControl.Item class="seg-opt" value={o.value}>
            <SegmentedControl.ItemInput />
            <SegmentedControl.ItemLabel>{o.label}</SegmentedControl.ItemLabel>
          </SegmentedControl.Item>
        )}
      </For>
    </SegmentedControl>
  )
}
