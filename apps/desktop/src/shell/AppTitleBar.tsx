import { DispatchPill } from "./DispatchPill";
import { InboxPill } from "./InboxPill";
import type { InboxItem } from "../fixtures/inbox";
import type { ActiveSession } from "./RunningPill";

export interface AppTitleBarProps {
    categoryLabel: string;
    viewLabel: string;
    running: number;
    needsYou: number;
    sessions?: ActiveSession[];
    inboxCount?: number;
    inboxItems?: readonly InboxItem[];
    onOpenDispatch?: () => void;
    onStopAll?: () => void;
    onOpenInbox?: () => void;
    onDispatchOpenChange?: (open: boolean) => void;
}

export function AppTitleBar(props: AppTitleBarProps) {
    return (
        <div class="titlebar" data-testid="app-titlebar">
            <div class="traffic" data-testid="traffic-lights">
                <span class="traffic-close" />
                <span class="traffic-min" />
                <span class="traffic-max" />
            </div>
            <div class="wordmark" data-testid="wordmark">
                Locus
            </div>
            <div class="title-context">
                <span data-testid="title-category">{props.categoryLabel}</span>
                <span data-testid="title-view">{props.viewLabel}</span>
            </div>
            <div style={{ flex: 1 }} />
            <DispatchPill
                running={props.running}
                needsYou={props.needsYou}
                sessions={props.sessions}
                onOpenDispatch={props.onOpenDispatch}
                onStopAll={props.onStopAll}
                onOpenChange={props.onDispatchOpenChange}
            />
            <InboxPill
                count={props.inboxCount ?? 0}
                items={props.inboxItems}
                onOpenInbox={props.onOpenInbox}
            />
        </div>
    );
}
