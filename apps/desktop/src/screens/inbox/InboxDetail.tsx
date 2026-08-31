import { Show, createSignal } from "solid-js";
import { Button } from "../../ui/Button";
import { Textarea } from "../../ui/Input";
import type { InboxDelivery } from "../../data/inbox";

export interface InboxDetailProps {
  item: InboxDelivery;
  /** Resolves the item in place. The comment is optional steering. The view does not change. */
  onApprove: (comment: string) => void;
  /** Returns the work with the comment as its reason. An empty comment never reaches here. */
  onSendBack: (comment: string) => void;
}

const age = (createdAt: string | null) => {
  if (!createdAt) return "an unknown time";
  const minutes = Math.max(
    0,
    Math.floor((Date.now() - Date.parse(createdAt)) / 60000),
  );
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  return hours < 24 ? `${hours}h` : `${Math.floor(hours / 24)}d`;
};

export function InboxDetail(props: InboxDetailProps) {
  const [comment, setComment] = createSignal("");
  const [sendBackBlocked, setSendBackBlocked] = createSignal(false);

  /**
   * Send back returns the work with its reason — the comment is the response,
   * not a note, so an empty one blocks the submit instead of resolving.
   * Approve does not ask for text; the two actions stay separate paths.
   */
  const requestSendBack = () => {
    if (!comment().trim()) {
      setSendBackBlocked(true);
      return;
    }
    setSendBackBlocked(false);
    props.onSendBack(comment());
  };

  return (
    <section class="inbox-detail" data-testid="inbox-detail">
      <header class="inbox-detail-head">
        <span class="inbox-detail-kind" data-testid="inbox-detail-kind">
          {props.item.senderKind === "agent" ? "Agent message" : "Message"}
        </span>
        <h1 class="inbox-detail-title" data-testid="inbox-detail-title">
          {props.item.subject}
        </h1>
        <div class="inbox-detail-meta" data-testid="inbox-detail-meta">
          <span>{props.item.project}</span>
          <span>·</span>
          <span class="mono">{props.item.senderKind}</span>
          <Show when={props.item.createdAt}>
            <span>·</span>
            <span>held {age(props.item.createdAt)}</span>
          </Show>
        </div>
      </header>

      <div class="inbox-detail-body">
        <div class="inbox-body-label" data-testid="inbox-body-label">
          Message
        </div>
        <div class="inbox-steps" data-testid="inbox-steps">
          {props.item.body}
        </div>

        <div class="inbox-comment">
          <span
            class="inbox-comment-caption"
            data-testid="inbox-comment-caption"
          >
            Comment steers the agent that made it
          </span>
          <Textarea
            data-testid="inbox-comment"
            value={comment()}
            placeholder="Optional"
            aria-invalid={sendBackBlocked() ? "true" : undefined}
            onInput={(e) => {
              setComment(e.currentTarget.value);
              if (sendBackBlocked()) setSendBackBlocked(false);
            }}
          />
          <Show when={sendBackBlocked()}>
            <p
              class="inbox-send-back-error"
              role="alert"
              data-testid="inbox-send-back-error"
              style={{
                margin: 0,
                color: "var(--status-danger)",
                "font-size": "var(--t-micro)",
              }}
            >
              Write a comment — send back returns the work with your reason.
            </p>
          </Show>
        </div>
        <div class="inbox-explanations" data-testid="inbox-explanations">
          <p data-testid="inbox-cost">
            <strong>Cost of waiting</strong>
            One loop held since{" "}
            {props.item.createdAt
              ? new Date(props.item.createdAt).toLocaleString()
              : "an unknown time"}
            . No tokens burn while blocked.
          </p>
        </div>
      </div>

      <footer class="inbox-footer" data-testid="inbox-footer">
        <Button
          variant="primary"
          data-testid="inbox-approve"
          onClick={() => props.onApprove(comment())}
        >
          Approve &amp; release the loop
        </Button>
        <Button
          variant="secondary"
          data-testid="inbox-send-back"
          onClick={requestSendBack}
        >
          Send back with comment
        </Button>
        <span class="inbox-footer-note" data-testid="inbox-footer-note">
          Resolves here · the decision is recorded on the thread
        </span>
      </footer>
    </section>
  );
}
