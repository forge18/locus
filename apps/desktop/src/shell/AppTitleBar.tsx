import { getCurrentWindow } from "@tauri-apps/api/window";
import { DispatchPill } from "./DispatchPill";
import { InboxPill } from "./InboxPill";
import type { InboxDelivery } from "../data/inbox";
import type { ActiveSession } from "./session-types";
import type { StoreHealth } from "../data/health";

export interface AppTitleBarProps {
    categoryLabel: string;
    viewLabel: string;
    running: number;
    needsYou: number;
    sessions?: ActiveSession[];
    inboxCount?: number;
    inboxItems?: readonly InboxDelivery[];
    onOpenDispatch?: () => void;
    onStopAll?: () => void;
    onOpenInbox?: () => void;
    onDispatchOpenChange?: (open: boolean) => void;
    storeHealth?: StoreHealth;
}

/** The custom title bar owns the window chrome: with native decorations off,
 * these controls are the only ones, so they must really work. */
export function AppTitleBar(props: AppTitleBarProps) {
    // Resolved per click, not per render: the component must mount outside a
    // Tauri runtime (tests, demo hosts) without touching the window API.
    const closeWindow = () => void getCurrentWindow().close();
    const minimizeWindow = () => void getCurrentWindow().minimize();
    const toggleZoom = () => void getCurrentWindow().toggleMaximize();
    return (
        <div class="titlebar" data-testid="app-titlebar" data-tauri-drag-region>
            <div class="traffic" data-testid="traffic-lights">
                <button
                    type="button"
                    class="traffic-close"
                    aria-label="Close window"
                    data-testid="window-close"
                    onClick={closeWindow}
                />
                <button
                    type="button"
                    class="traffic-min"
                    aria-label="Minimize window"
                    data-testid="window-minimize"
                    onClick={minimizeWindow}
                />
                <button
                    type="button"
                    class="traffic-max"
                    aria-label="Toggle window zoom"
                    data-testid="window-maximize"
                    onClick={toggleZoom}
                />
            </div>
            <div class="wordmark" data-testid="wordmark" data-tauri-drag-region>
                Locus
            </div>
            <div class="title-context" data-tauri-drag-region>
                <span data-testid="title-category">{props.categoryLabel}</span>
                <span data-testid="title-view">{props.viewLabel}</span>
            </div>
            <div style={{ flex: 1 }} data-tauri-drag-region />
            <span
                data-testid="store-health"
                data-status={props.storeHealth?.status ?? "not_configured"}
                title={props.storeHealth?.message ?? undefined}
            >
                {props.storeHealth?.status ?? "not configured"}
            </span>
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
