import type { InboxDelivery } from "../../data/inbox";

export interface InboxCardProps {
    item: InboxDelivery;
    selected: boolean;
    onSelect: () => void;
}

const age = (createdAt: string | null) => {
    if (!createdAt) return "—";
    const minutes = Math.max(
        0,
        Math.floor((Date.now() - Date.parse(createdAt)) / 60000),
    );
    return minutes < 60 ? `${minutes}m` : `${Math.floor(minutes / 60)}h`;
};

export function InboxCard(props: InboxCardProps) {
    return (
        <button
            type="button"
            class="inbox-card"
            data-testid={`inbox-card-${props.item.id}`}
            aria-selected={props.selected ? "true" : "false"}
            onClick={props.onSelect}
        >
            <div class="inbox-card-head">
                <span class="inbox-card-title">{props.item.subject}</span>
                <span class="inbox-card-age" data-testid="inbox-card-age">
                    {age(props.item.createdAt)}
                </span>
            </div>
            <div class="inbox-card-sub" data-testid="inbox-card-sub">
                {props.item.project} ·{" "}
                <span class="mono">{props.item.senderKind}</span>
            </div>
        </button>
    );
}
