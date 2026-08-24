import { For, Show, createSignal } from "solid-js";
import type { InboxItem } from "../fixtures/inbox";

export interface InboxPillProps {
    count: number;
    items?: readonly InboxItem[];
    onOpenInbox?: () => void;
}

export function InboxPill(props: InboxPillProps) {
    const [open, setOpen] = createSignal(false);
    return (
        <div class="title-pill-wrap">
            <button
                type="button"
                class="title-pill"
                data-testid="inbox-pill"
                aria-expanded={open()}
                onClick={() => setOpen(!open())}
            >
                <span aria-hidden="true">▱</span>
                <span>Inbox</span>
                <Show when={props.count > 0}>
                    <span class="title-pill-badge">{props.count}</span>
                </Show>
            </button>
            <Show when={open()}>
                <div
                    class="activity-popover inbox-popover"
                    role="dialog"
                    aria-label="Inbox preview"
                    data-testid="inbox-popover"
                >
                    <p>
                        {props.count
                            ? `${props.count} items need a response.`
                            : "Nothing needs a response."}
                    </p>
                    <Show when={props.items?.length}>
                        <ul data-testid="inbox-preview-items">
                            <For each={props.items}>
                                {(item) => (
                                    <li
                                        data-testid={`inbox-preview-${item.id}`}
                                    >
                                        <strong>{item.title}</strong>
                                        <small>
                                            {item.project} · {item.agent}
                                        </small>
                                    </li>
                                )}
                            </For>
                        </ul>
                    </Show>
                    <footer>
                        <button
                            type="button"
                            onClick={() => {
                                setOpen(false);
                                props.onOpenInbox?.();
                            }}
                        >
                            Open Inbox
                        </button>
                    </footer>
                </div>
            </Show>
        </div>
    );
}
