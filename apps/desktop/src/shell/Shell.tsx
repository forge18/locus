import { createSignal, onCleanup, onMount, Show } from "solid-js";
import type { JSX } from "solid-js";
import { AppTitleBar } from "./AppTitleBar";
import { ProjectRail } from "./ProjectRail";
import {
    LocatorPalette,
    type PaletteSessionState,
} from "../nav/LocatorPalette";
import { Sheet } from "../ui/Sheet";
import { notify, ToastRegion } from "../ui/Toast";
import { useRunningCount, useStripCards } from "../data/strip";
import { stopAllDispatch } from "../data/dispatch";
import { useInboxItems } from "../data/inbox";
import type { ActiveSession } from "./RunningPill";
import type { NavStore, View } from "../nav";
import { destinationDesktop } from "../nav/desktop-navigation";
import type { DesktopNavTarget, DesktopRouteId } from "../nav/desktop-locator";
import {
    Desktop_PROJECT_ROUTE_KINDS,
    Desktop_ROUTE_KINDS,
} from "../nav/desktop-route-kinds";

const Desktop_ROUTE_IDS = Desktop_ROUTE_KINDS.map((route) => route.id);

export interface ShellProps {
    nav: NavStore;
    children: JSX.Element;
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
    return Desktop_PROJECT_ROUTE_KINDS.some(
        (candidate) => candidate.id === route,
    )
        ? destinationDesktop(
              route,
              project,
              view === "bots" ? botId : undefined,
          )
        : destinationDesktop(route);
}

/** The desktop title bar and project-scoped rail frame every screen. */
export function Shell(props: ShellProps) {
    const [paletteOpen, setPaletteOpen] = createSignal(false);
    const [dispatchOpen, setDispatchOpen] = createSignal(false);
    const inboxItems = useInboxItems();
    const activeSessions: ActiveSession[] = useStripCards()
        .filter((card) => card.kind === "agent")
        .map((card) => ({
            id: card.id,
            label: `${card.project} · ${card.agent}`,
            needsAttention:
                card.status === "waiting" || card.status === "stuck",
            lastActivityAt: card.idleMinutes,
            project: card.project,
            role: card.role ?? undefined,
            elapsed:
                card.idleMinutes === 0 ? "now" : `${card.idleMinutes}m ago`,
            meta: card.status ?? card.tool ?? "running",
        }));
    const needsYou = activeSessions.filter(
        (session) => session.needsAttention,
    ).length;
    const paletteSessions: PaletteSessionState[] = activeSessions.map(
        (session) => ({
            project: session.project ?? "tapestry",
            needsAttention: session.needsAttention,
        }),
    );
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
    const currentDesktopLocator = () =>
        desktopLocatorFor(
            props.nav.view(),
            props.nav.params().project ?? "tapestry",
            props.nav.params().botId,
        );

    // ⌘K resolves a locator. It is bound here because the palette is shell
    // chrome, and there is one of it per window.
    const onKeyDown = (e: KeyboardEvent) => {
        if (
            (e.key.toLowerCase() === "k" || e.key.toLowerCase() === "p") &&
            (e.metaKey || e.ctrlKey)
        ) {
            e.preventDefault();
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
                running={useRunningCount()}
                needsYou={needsYou}
                sessions={activeSessions}
                inboxCount={inboxItems.length}
                inboxItems={inboxItems}
                onOpenDispatch={() =>
                    openDesktopLocator(destinationDesktop("autorun"))
                }
                onStopAll={() => {
                    void stopAllDispatch().then(
                        ({ stoppedRuns }) =>
                            notify({
                                title: "Dispatch stopped",
                                description: `${stoppedRuns} run${stoppedRuns === 1 ? "" : "s"} stopped.`,
                            }),
                    ).catch((error: unknown) =>
                        notify({
                            title: "Stop all failed",
                            description: error instanceof Error ? error.message : String(error),
                            type: "error",
                        }),
                    );
                }}
                onOpenInbox={() =>
                    openDesktopLocator(destinationDesktop("inbox"))
                }
                onDispatchOpenChange={setDispatchOpen}
            />
            <div class="body">
                <ProjectRail
                    selectedProject={props.nav.params().project ?? "tapestry"}
                    onNavigate={openDesktopLocator}
                />
                <div class="main">
                    <div class="screen" data-testid="screen">
                        {props.children}
                    </div>
                </div>
            </div>

            <Show when={props.nav.view() !== "interact" && !dispatchOpen()}>
                <ToastRegion />
            </Show>
            <LocatorPalette
                open={paletteOpen()}
                onOpenChange={setPaletteOpen}
                current={currentDesktopLocator()}
                project={props.nav.params().project ?? "tapestry"}
                history={props.nav.history()}
                sessions={paletteSessions}
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
