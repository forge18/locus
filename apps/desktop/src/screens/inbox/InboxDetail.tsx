import { For, Show, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { Textarea } from '../../ui/Input'
import type { InboxItem } from '../../data/inbox'

export interface InboxDetailProps {
  item: InboxItem
  /** Resolves the item in place. The view does not change. */
  onApprove: (comment: string) => void
  onSendBack: (comment: string) => void
  /** Opens the item's work where that work lives, by locator. */
  onOpenWork: (locator: string) => void
}

const BODY_LABEL: Record<InboxItem['kind'], string> = {
  gate: 'Plan',
  ask: 'Question',
  guardrail: 'What happened',
}

export function InboxDetail(props: InboxDetailProps) {
  const [comment, setComment] = createSignal('')

  return (
    <section class="inbox-detail" data-testid="inbox-detail">
      <header class="inbox-detail-head">
        <span class="inbox-detail-kind" data-testid="inbox-detail-kind">
          {props.item.kind}
        </span>
        <h1 class="inbox-detail-title" data-testid="inbox-detail-title">
          {props.item.title}
        </h1>
        <div class="inbox-detail-meta" data-testid="inbox-detail-meta">
          {/* The locator is the affordance: opening the work is going to its address. */}
          <button
            type="button"
            class="inbox-detail-locator"
            data-testid="inbox-detail-locator"
            data-open-work="true"
            onClick={() => props.onOpenWork(props.item.opensAt)}
          >
            {props.item.opensAt}
          </button>
          <span>·</span>
          <span>{props.item.agent}</span>
          <span>·</span>
          <span>{props.item.role}</span>
          <span>·</span>
          <span>Gate: {props.item.kind === 'gate' ? 'human' : 'agent'}</span>
        </div>
      </header>

      <div class="inbox-detail-body">
        <div class="inbox-body-label" data-testid="inbox-body-label">
          {BODY_LABEL[props.item.kind]}
        </div>
        <ol class="inbox-steps" data-testid="inbox-steps">
          <For each={props.item.body}>
            {(step) => <li innerHTML={monoPaths(step)} />}
          </For>
        </ol>

        <Show when={props.item.callout}>
          <div class="inbox-callout" data-testid="inbox-callout">
            <Icon name="info" size={12} style={{ 'flex-shrink': 0, 'margin-top': '1px' }} />
            <span>{props.item.callout}</span>
          </div>
        </Show>

        <div class="inbox-comment">
          <span class="inbox-comment-caption" data-testid="inbox-comment-caption">
            Comment steers the agent that made it
          </span>
          <Textarea
            data-testid="inbox-comment"
            value={comment()}
            placeholder="Optional"
            onInput={(e) => setComment(e.currentTarget.value)}
          />
        </div>
      </div>

      <footer class="inbox-footer" data-testid="inbox-footer">
        <Button variant="primary" data-testid="inbox-approve" onClick={() => props.onApprove(comment())}>
          Approve &amp; release the loop
        </Button>
        <Button variant="secondary" data-testid="inbox-send-back" onClick={() => props.onSendBack(comment())}>
          Send back with comment
        </Button>
        <span class="inbox-footer-note" data-testid="inbox-footer-note">
          Resolves here · the work opens where the work lives
        </span>
      </footer>
    </section>
  )
}

/**
 * Backtick spans become mono, which is how an inline path reads as a path. The
 * source is fixture prose we author, not anything a user or an agent typed.
 */
function monoPaths(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/`([^`]+)`/g, '<code class="mono">$1</code>')
}
