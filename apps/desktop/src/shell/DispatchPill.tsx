import { For, Show, createSignal } from "solid-js";
import type { ActiveSession } from "./RunningPill";

export interface DispatchPillProps {
  running: number;
  needsYou: number;
  sessions?: ActiveSession[];
  onOpenDispatch?: () => void;
  onStopAll?: () => void;
  onOpenChange?: (open: boolean) => void;
}

export function DispatchPill(props: DispatchPillProps) {
  const [open, setOpen] = createSignal(false);
  const [tab, setTab] = createSignal<"attention" | "all">("attention");
  const visibleSessions = () =>
    (props.sessions ?? []).filter(
      (session) => tab() === "all" || session.needsAttention,
    );
  const elapsed = (session: ActiveSession) =>
    session.elapsed ??
    (session.lastActivityAt <= 0 ? "now" : `${session.lastActivityAt}m ago`);
  return (
    <div class="title-pill-wrap">
      <button
        type="button"
        class="title-pill"
        data-testid="dispatch-pill"
        aria-expanded={open()}
        onClick={() => {
          const next = !open();
          setOpen(next);
          props.onOpenChange?.(next);
        }}
      >
        <span aria-hidden="true">◉</span>
        <span>{props.running}</span>
        <Show when={props.running > 0}>
          <span class="live-dot pulse" />
        </Show>
      </button>
      <Show when={open()}>
        <div
          class="activity-popover"
          role="dialog"
          aria-label="Dispatch activity"
          data-testid="dispatch-popover"
        >
          <div class="activity-tabs">
            <button
              type="button"
              aria-selected={tab() === "attention"}
              onClick={() => setTab("attention")}
            >
              Attention needed
            </button>
            <button
              type="button"
              aria-selected={tab() === "all"}
              onClick={() => setTab("all")}
            >
              All
            </button>
          </div>
          <p>
            {tab() === "attention"
              ? "Three runs are blocked on you. Everything else is running and does not need a decision."
              : "Runs first, then what has already happened. Nothing here is an obligation unless it is in the first group."}
          </p>
          <ul data-testid="dispatch-activity-list">
            <For each={visibleSessions()}>
              {(session) => (
                <li data-testid={`dispatch-activity-${session.id}`}>
                  <span aria-hidden="true">
                    {session.needsAttention ? "!" : "◉"}
                  </span>
                  <strong>{session.label}</strong>
                  <small>{elapsed(session)}</small>
                  <span class="dispatch-project-tag">
                    {session.project ?? "project"}
                  </span>
                  <small>{session.meta ?? session.role ?? "running"}</small>
                </li>
              )}
            </For>
          </ul>
          <footer>
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                props.onStopAll?.();
              }}
            >
              Stop all
            </button>
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                props.onOpenDispatch?.();
              }}
            >
              Open Dispatch
            </button>
          </footer>
        </div>
      </Show>
    </div>
  );
}
