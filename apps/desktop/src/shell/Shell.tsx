import { createMemo, createSignal, onCleanup, onMount, Show } from "solid-js";
import type { JSX } from "solid-js";
import { AppTitleBar } from "./AppTitleBar";
import { ProjectRail } from "./ProjectRail";
import { TabBar } from "./TabBar";
import {
    LocatorPalette,
    type PaletteMode,
    type PaletteResult,
    type PaletteSessionState,
} from "../nav/LocatorPalette";
import { Sheet } from "../ui/Sheet";
import { notify, ToastRegion } from "../ui/Toast";
import {
    fetchRunningCount,
    fetchStripCards,
    type StripCard,
} from "../data/strip";
import { stopAllDispatch } from "../data/dispatch";
import { fetchInboxPendingCount } from "../data/inbox";
import { fetchStoreHealth, type StoreHealth } from "../data/health";
import type { Envelope } from "../data/envelope";
import type { ActiveSession } from "./session-types";
import { BackLink, type NavStore, type View } from "../nav";
import { destinationDesktop } from "../nav/desktop-navigation";
import type { DesktopNavTarget, DesktopRouteId } from "../nav/desktop-locator";
import { Desktop_ROUTE_KINDS } from "../nav/desktop-route-kinds";

const Desktop_ROUTE_IDS = Desktop_ROUTE_KINDS.map((route) => route.id);

function safeLiveRead<T>(
    command: string,
    read: () => Promise<Envelope<T>>,
): Promise<Envelope<T>> {
    return Promise.resolve()
        .then(read)
        .catch((cause) => ({
            status: "failed" as const,
            error: {
                command,
                message: cause instanceof Error ? cause.message : String(cause),
            },
        }));
}

export interface ShellProps {
    nav: NavStore;
    children: JSX.Element;
    /** Unified search_all results supplied by the command surface. */
    searchAll?: (
        query: string,
    ) => PaletteResult[] | Promise<PaletteResult[]>;
}

const desktopViews: Record<DesktopRouteId, View> = Object.fromEntries(
    Desktop_ROUTE_IDS.map((route) => [route, route]),
) as Record<DesktopRouteId, View>;

const desktopRoutes: Record<View, DesktopRouteId> = Object.fromEntries(
    Desktop_ROUTE_IDS.map((route) => [route, route]),
) as Record<View, DesktopRouteId>;

/** Maps every registered desktop route to the currently delivered shared surface. */
export function desktopViewFor(target: DesktopNavTarget): View {
    return desktopViews[target.route];
}

/** Maps a delivered shared surface back to its canonical desktop route. */
export function desktopLocatorFor(
    view: View,
    project: string,
    botId?: string,
): string {
    const route = desktopRoutes[view];
    if (view === "workers" && botId)
        return destinationDesktop(route, project, botId);
    return destinationDesktop(route);
}

/** The desktop title bar and project-scoped rail frame every screen. */
export function Shell(props: ShellProps) {
    const [paletteOpen, setPaletteOpen] = createSignal(false);
    const [paletteMode, setPaletteMode] = createSignal<PaletteMode>("locator");
    const [dispatchOpen, setDispatchOpen] = createSignal(false);
    const [stripEnvelope, setStripEnvelope] = createSignal<
        Envelope<StripCard[]>
    >({ status: "loading" });
    const [runningEnvelope, setRunningEnvelope] = createSignal<
        Envelope<number>
    >({ status: "loading" });
    const [inboxEnvelope, setInboxEnvelope] = createSignal<Envelope<number>>({
        status: "loading",
    });
    const [healthEnvelope, setHealthEnvelope] = createSignal<
        Envelope<StoreHealth>
    >({ status: "loading" });

    const loadLiveStatus = async () => {
        const [cards, running, inbox, health] = await Promise.all([
            safeLiveRead("strip_cards", fetchStripCards),
            safeLiveRead("running_count", fetchRunningCount),
            safeLiveRead("inbox_pending_count", fetchInboxPendingCount),
            safeLiveRead("store_health", fetchStoreHealth),
        ]);
        setStripEnvelope(cards);
        setRunningEnvelope(running);
        setInboxEnvelope(inbox);
        setHealthEnvelope(health);
        for (const failed of [cards, running, inbox, health]) {
            if (failed.status === "failed") {
                notify({
                    title: "Live status unavailable",
                    description: failed.error.message,
                    type: "error",
                });
            }
        }
    };
    onMount(() => {
        void loadLiveStatus().catch((cause) => {
            notify({
                title: "Live status unavailable",
                description:
                    cause instanceof Error ? cause.message : String(cause),
                type: "error",
            });
        });
    });

    const activeSessions = createMemo<ActiveSession[]>(() => {
        const envelope = stripEnvelope();
        if (envelope.status !== "ready") return [];
        return envelope.data.map((card) => ({
            id: card.id,
            label: `${card.project} · ${card.agent}`,
            needsAttention:
                card.status === "waiting" || card.status === "stuck",
            lastActivityAt: card.idleMinutes,
            project: card.project,
            elapsed:
                card.idleMinutes === 0 ? "now" : `${card.idleMinutes}m ago`,
            meta: card.status ?? "running",
        }));
    });
    const needsYou = createMemo(
        () =>
            activeSessions().filter((session) => session.needsAttention).length,
    );
    const runningCount = createMemo(() => {
        const envelope = runningEnvelope();
        return envelope.status === "ready" ? envelope.data : 0;
    });
    const inboxCount = createMemo(() => {
        const envelope = inboxEnvelope();
        return envelope.status === "ready" ? envelope.data : 0;
    });
    const paletteSessions = createMemo<PaletteSessionState[]>(() =>
        activeSessions().map((session) => ({
            project: session.project ?? "tapestry",
            needsAttention: session.needsAttention,
        })),
    );
    const storeHealth = createMemo<StoreHealth | undefined>(() => {
        const envelope = healthEnvelope();
        return envelope.status === "ready" ? envelope.data : undefined;
    });
    const openDesktopTarget = (target: DesktopNavTarget) => {
        const params =
            target.scope.kind === "project"
                ? {
                      project: target.scope.project,
                      ...(target.botId ? { botId: target.botId } : {}),
                  }
                : { project: undefined };
        props.nav.go(desktopViewFor(target), params);
    };
    const openDesktopLocator = (locator: string) => props.nav.open(locator);
    const currentDesktopLocator = () => props.nav.locator();

    // ⌘K resolves a locator. It is bound here because the palette is shell
    // chrome, and there is one of it per window.
    const onKeyDown = (e: KeyboardEvent) => {
        if (
            (e.key.toLowerCase() === "k" || e.key.toLowerCase() === "p") &&
            (e.metaKey || e.ctrlKey)
        ) {
            e.preventDefault();
            setPaletteMode(e.key.toLowerCase() === "p" ? "search" : "locator");
            setPaletteOpen(true);
        }
    };
    onMount(() => document.addEventListener("keydown", onKeyDown));
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));

    return (
        <div class="window" data-testid="window">
            <AppTitleBar
                categoryLabel={props.nav.categoryLabel()}
                viewLabel={props.nav.view()}
                running={runningCount()}
                needsYou={needsYou()}
                sessions={activeSessions()}
                inboxCount={inboxCount()}
                onOpenDispatch={() =>
                    openDesktopLocator(destinationDesktop("autorun"))
                }
                onStopAll={() => {
                    void stopAllDispatch()
                        .then(({ stoppedRuns }) =>
                            notify({
                                title: "Dispatch stopped",
                                description: `${stoppedRuns} run${stoppedRuns === 1 ? "" : "s"} stopped.`,
                            }),
                        )
                        .catch((error: unknown) =>
                            notify({
                                title: "Stop all failed",
                                description:
                                    error instanceof Error
                                        ? error.message
                                        : String(error),
                                type: "error",
                            }),
                        );
                }}
                onOpenInbox={() =>
                    openDesktopLocator(destinationDesktop("inbox"))
                }
                onDispatchOpenChange={setDispatchOpen}
                storeHealth={storeHealth()}
            />
            <TabBar
                view={props.nav.view()}
                locator={props.nav.locatorPath()}
                onNavigate={(view) => props.nav.go(view)}
            />
            <div class="body">
                <ProjectRail
                    onNavigate={openDesktopLocator}
                />
                <div class="main">
                    {/* A drill-down's way out is the view it was entered from,
                        so the back link rides above the screen, shell-owned. It
                        renders only when the current view is one. */}
                    <BackLink nav={props.nav} />
                    <div class="screen" data-testid="screen">
                        {props.children}
                    </div>
                </div>
            </div>

            <Show when={!dispatchOpen()}>
                <ToastRegion />
            </Show>
            <LocatorPalette
                open={paletteOpen()}
                onOpenChange={setPaletteOpen}
                current={currentDesktopLocator()}
                project={props.nav.params().project}
                history={props.nav.history()}
                sessions={paletteSessions()}
                mode={paletteMode()}
                searchAll={props.searchAll}
                onResolve={openDesktopTarget}
                onOpenLocator={openDesktopLocator}
            />
            <Sheet
                open={props.nav.detail() !== null}
                onOpenChange={(open) => !open && props.nav.closeDetail()}
                title={props.nav.detail() ? props.nav.detail()!.view : ""}
            >
                <p class="t-body" data-testid="detail-body">
                    {props.nav.detail()
                        ? JSON.stringify(props.nav.detail()!.params)
                        : ""}
                </p>
            </Sheet>
        </div>
    );
}
