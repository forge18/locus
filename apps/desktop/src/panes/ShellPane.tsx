import { Terminal } from "@xterm/xterm";
import { Channel, invoke } from "@tauri-apps/api/core";
import { createSignal, onCleanup, onMount, Show } from "solid-js";
import { InlineError } from "../ui/InlineError";
import { coalesce } from "./coalesce";
import { reachesTerminal, terminalOptions } from "./shell-config";

export interface ShellPaneProps {
  runId: string;
  title?: string;
}

/** A PTY-backed pane. It owns an xterm instance; AgentPane deliberately does not. */
export function ShellPane(props: ShellPaneProps) {
  let host: HTMLDivElement | undefined;
  let terminal: Terminal | undefined;
  const [streamError, setStreamError] = createSignal<string | null>(null);

  onMount(() => {
    terminal = new Terminal(terminalOptions);
    terminal.attachCustomKeyEventHandler(reachesTerminal);
    terminal.open(host!);
    const frames = coalesce<Uint8Array>((chunks) =>
      chunks.forEach((chunk) => terminal?.write(chunk)),
    );
    const channel = new Channel<number[]>();
    let stopped = false;
    channel.onmessage = (bytes) => frames.push(Uint8Array.from(bytes));
    onCleanup(() => {
      stopped = true;
      channel.onmessage = () => undefined;
      frames.stop();
      terminal?.dispose();
    });
    void invoke("pty_subscribe", { runId: props.runId, channel }).catch(
      (error: unknown) => {
        if (!stopped)
          setStreamError(
            error instanceof Error ? error.message : String(error),
          );
      },
    );
  });

  return (
    <section
      class="shell-pane"
      data-testid="shell-pane"
      data-run-id={props.runId}
      data-pty="true"
    >
      <Show when={streamError()}>
        {(error) => (
          <InlineError
            cause={error()}
            next="Check the core PTY stream and reopen this pane."
          />
        )}
      </Show>
      <div class="shell-pane-terminal" ref={host} />
    </section>
  );
}
