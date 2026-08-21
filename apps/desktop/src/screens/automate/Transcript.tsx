import { For } from 'solid-js'
import { EVENT_VERBS } from '../../types/event'
import type { EventVerb } from '../../types/event'
import type { SessionDetail } from '../../data/sessions'

export interface TranscriptProps {
  session: SessionDetail
}

/**
 * Colored by verb, and only by verb. The palette is keyed on the twelve canonical
 * verbs, so a thirteenth has nowhere to land — which is the point: a source that
 * cannot report `thinking` produces no `thinking` line rather than an uncoloured one.
 */
export const VERB_CLASS: Record<EventVerb, string> = Object.fromEntries(
  EVENT_VERBS.map((verb) => [verb, `verb-${verb}`]),
) as Record<EventVerb, string>

export function Transcript(props: TranscriptProps) {
  return (
    <div class="transcript-body" data-testid="transcript">
      <For each={props.session.transcript}>
        {(line) => (
          <div
            class={`transcript-line ${VERB_CLASS[line.verb]}`}
            data-verb={line.verb}
            data-testid={`transcript-line-${line.verb}`}
          >
            {line.text}
          </div>
        )}
      </For>
      <div class="transcript-prompt" data-testid="transcript-prompt">
        <span>{props.session.prompt}</span>
        <span class="transcript-cursor blink" data-testid="transcript-cursor" />
      </div>
    </div>
  )
}
