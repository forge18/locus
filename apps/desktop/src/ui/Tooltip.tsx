import { Tooltip as KTooltip } from '@kobalte/core/tooltip'
import type { JSX } from 'solid-js'

export interface TooltipProps {
  /** The text to show. Keep it to a fact the trigger cannot fit. */
  content: JSX.Element
  children: JSX.Element
  placement?: 'top' | 'bottom' | 'left' | 'right'
  openDelay?: number
}

export function Tooltip(props: TooltipProps) {
  return (
    <KTooltip placement={props.placement ?? 'top'} openDelay={props.openDelay ?? 400} gutter={6}>
      <KTooltip.Trigger as="span" data-testid="tooltip-trigger">
        {props.children}
      </KTooltip.Trigger>
      <KTooltip.Portal>
        <KTooltip.Content class="tooltip" data-testid="tooltip-content">
          {props.content}
        </KTooltip.Content>
      </KTooltip.Portal>
    </KTooltip>
  )
}
