import { For, Show } from "solid-js";
import type { StripCard } from "../data/strip";

export interface StripProps {
  cards: StripCard[];
}

const tokens = (n: number | null) =>
  // Unknown is not zero. A harness that reports nothing gets the word, not a number.
  n === null ? "unknown" : n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);

/**
 * The running-agent footer. It persists across categories: leaving Manage is not
 * a reason to lose sight of what is running.
 */
export function Strip(props: StripProps) {
  return (
    <footer class="strip" data-testid="strip">
      <span class="strip-label" data-testid="strip-label">
        Strip
      </span>
      <div class="strip-cards">
        <For each={props.cards}>
          {(card) => (
            <div
              class={[
                "strip-card",
                card.status === "stuck" ? "strip-card-stuck" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              data-testid={`strip-card-${card.id}`}
              data-kind={card.kind}
              data-status={card.status ?? undefined}
              data-task-id={card.taskId}
              data-task-locator={
                card.taskId
                  ? `locus://${card.project}/task/${card.taskId}`
                  : undefined
              }
            >
              <div class="strip-card-top">
                {`${card.project} · ${card.agent} · ${card.role}`}
              </div>
              <div class="strip-card-bottom">
                <Show when={card.taskId}>
                  <a
                    class="strip-task-link"
                    href={`locus://${card.project}/task/${card.taskId}`}
                    data-testid={`strip-task-${card.taskId}`}
                  >
                    task
                  </a>
                </Show>
                {`${card.status} · ${card.tool ?? "no tool"} · ${tokens(card.tokens)}`}
              </div>
            </div>
          )}
        </For>
      </div>
      <span class="strip-note">sorted by needs-attention, then activity</span>
    </footer>
  );
}
