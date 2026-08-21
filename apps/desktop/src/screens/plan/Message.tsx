import { For, Show } from 'solid-js'
import type { PlanMessage } from '../../data/plan'

export interface MessageProps {
  message: PlanMessage
}

/**
 * Three speakers that have to be told apart without reading: an agent on the blue
 * ground, the auditor on the deep amber one with a red-tinted bubble, and you,
 * right-aligned on the raised surface.
 */
export function Message(props: MessageProps) {
  const speaker = () => props.message.speaker
  const isYou = () => speaker() === 'you'

  return (
    <div
      class={`msg msg-${speaker()}`}
      data-testid={`msg-${props.message.id}`}
      data-speaker={speaker()}
    >
      <Show when={!isYou()}>
        <div
          class={['msg-avatar', speaker() === 'auditor' ? 'msg-avatar-auditor' : '']
            .filter(Boolean)
            .join(' ')}
          data-testid={`msg-avatar-${props.message.id}`}
        >
          {props.message.initials}
        </div>
      </Show>
      <div class="msg-col">
        <span class="msg-caption" data-testid={`msg-caption-${props.message.id}`}>
          {isYou() ? 'you' : props.message.caption}
        </span>
        <Show when={props.message.facts.length > 0}>
          <div class="msg-facts" data-testid={`msg-facts-${props.message.id}`}>
            <For each={props.message.facts}>{(fact) => <span>{fact}</span>}</For>
          </div>
        </Show>
        <div class="msg-bubble" data-testid={`msg-bubble-${props.message.id}`}>
          {props.message.body}
        </div>
        <Show when={props.message.finding}>
          <span class="msg-finding" data-testid={`msg-finding-${props.message.id}`}>
            {props.message.finding}
          </span>
        </Show>
      </div>
    </div>
  )
}
