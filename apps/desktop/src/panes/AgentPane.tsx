import { createSignal, onCleanup, onMount, Show } from "solid-js";
import { InlineError } from "../ui/InlineError";
import { coalesce } from "./coalesce";
import { streamFromCore } from "../transcript/from-core";
import type { AgentEvent } from "../types/event";

export interface AgentPaneProps {
  runId: string;
}
export const agentPaneTransport = "event-channel" as const;

/** A typed telemetry transcript. This component never opens a PTY. */
export function AgentPane(props: AgentPaneProps) {
  const [events, setEvents] = createSignal<AgentEvent[]>([]);
  const [streamError, setStreamError] = createSignal<string | null>(null);
  let stopped = false;
  onMount(() => {
    const frames = coalesce<AgentEvent>((items) =>
      setEvents((current) => [...current, ...items]),
    );
    let detach = () => undefined;
    onCleanup(() => {
      stopped = true;
      frames.stop();
      detach();
    });
    void streamFromCore((event) => {
      if (!stopped && event.runId === props.runId) frames.push(event);
    })
      .then((channel) => {
        detach = () => {
          channel.onmessage = () => undefined;
        };
        if (stopped) detach();
      })
      .catch((error: unknown) => {
        if (!stopped)
          setStreamError(
            error instanceof Error ? error.message : String(error),
          );
      });
  });
  return (
    <section
      class="agent-pane"
      data-testid="agent-pane"
      data-run-id={props.runId}
      data-pty="false"
    >
      <Show when={streamError()}>
        {(error) => (
          <InlineError
            cause={error()}
            next="Check the core telemetry stream and reopen this pane."
          />
        )}
      </Show>
      <ol>
        {events().map((event) => (
          <li>{event.verb}</li>
        ))}
      </ol>
    </section>
  );
}
