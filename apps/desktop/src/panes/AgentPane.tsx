import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  untrack,
} from "solid-js";
import { InlineError } from "../ui/InlineError";
import { coalesce } from "./coalesce";
import { replayRunEvents, streamFromCore } from "../transcript/from-core";
import type { AgentEvent } from "../types/event";
import type {
  AgentGateMode,
  AgentPaneBlocker,
  AgentPaneCitation,
  AgentPaneFinding,
  AgentPanePlan,
  AgentPaneProps,
  AgentPaneSession,
  AgentPanelStatus,
  AgentThinkingDisplay,
  AgentToolDisplay,
} from "./agent-panel-model";
import { AgentHeader, Composer } from "./agent-pane-controls";
import { AgentEventStream } from "./agent-pane-content";
import {
  BlockerStack,
  CheckpointMarkers,
  PlanDock,
  ResearchPane,
} from "./agent-pane-docks";
import {
  eventText,
  eventsForRun,
  formatTokens,
  mergeEvents,
  mergeFindings,
} from "./agent-pane-utils";
import "./agent-pane.css";

export type {
  AgentFieldValue,
  AgentGateMode,
  AgentPaneBlocker,
  AgentPaneCheckpoint,
  AgentPaneCitation,
  AgentPaneElicitation,
  AgentPaneElicitationField,
  AgentPaneFinding,
  AgentPanePlan,
  AgentPanePlanStep,
  AgentPaneProps,
  AgentPaneSession,
  AgentPaneViewModel,
  AgentPanelStatus,
  AgentPermissionPosture,
  AgentThinkingDisplay,
  AgentToolDisplay,
} from "./agent-panel-model";

export const agentPaneTransport = "event-channel" as const;

const DEFAULT_SESSION: AgentPaneSession = {
  project: "tapestry",
  task: "Thread the notification channel",
  workflow: "build-and-verify · v4",
  agent: "builder@4",
  model: "claude-sonnet-4",
  harness: "pi",
  effort: "high",
  name: "Thread the notification channel",
  context: { used: 12_400, total: 200_000 },
  cost: "$0.42",
  permissionPosture: "bypass",
  status: "working",
};

export function AgentPane(props: AgentPaneProps) {
  const [events, setEvents] = createSignal<AgentEvent[]>(props.events ?? []);
  const [providedEvents, setProvidedEvents] = createSignal<AgentEvent[]>(
    props.events ?? [],
  );
  const [streamEvents, setStreamEvents] = createSignal<AgentEvent[]>([]);
  const [streamError, setStreamError] = createSignal<string | null>(null);
  const [internalResearchOpen, setInternalResearchOpen] = createSignal(false);
  const [internalCostVisible, setInternalCostVisible] = createSignal(false);
  const [contextOpen, setContextOpen] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [internalThinkingDisplay, setInternalThinkingDisplay] =
    createSignal<AgentThinkingDisplay>("summary");
  const [internalToolCallsDisplay, setInternalToolCallsDisplay] =
    createSignal<AgentToolDisplay>("expanded");
  const [internalGateMode, setInternalGateMode] =
    createSignal<AgentGateMode>("manual");
  const [composerValue, setComposerValue] = createSignal("");
  const [sessionName, setSessionName] = createSignal("");
  const [nameEdited, setNameEdited] = createSignal(false);
  const [localFindings, setLocalFindings] = createSignal<AgentPaneFinding[]>(
    [],
  );
  const [pinnedFindings, setPinnedFindings] = createSignal<AgentPaneFinding[]>(
    [],
  );
  const [reviewedFindings, setReviewedFindings] = createSignal<string[]>([]);
  const [restoredCheckpoint, setRestoredCheckpoint] = createSignal<
    string | null
  >(null);
  const [minimizedBlockers, setMinimizedBlockers] = createSignal<string[]>([]);
  const [resolvedBlockers, setResolvedBlockers] = createSignal<string[]>([]);
  let pane: HTMLElement | undefined;

  createEffect(
    on(
      () => `${props.runId}:${props.live === false ? "fixture" : "live"}`,
      () => {
        const runId = props.runId;
        const snapshot = untrack(() => eventsForRun(props.events ?? [], runId));
        setProvidedEvents(snapshot);
        setStreamEvents([]);
        setEvents(snapshot);
        setStreamError(null);
        setInternalThinkingDisplay("summary");
        setInternalToolCallsDisplay("expanded");
        setInternalGateMode("manual");
        setComposerValue("");
        setSessionName(
          untrack(() => props.session?.name ?? DEFAULT_SESSION.name),
        );
        setNameEdited(false);
        setPinnedFindings([]);
        setLocalFindings(untrack(() => props.findings ?? []));
        setReviewedFindings([]);
        setRestoredCheckpoint(null);
        setMinimizedBlockers([]);
        setResolvedBlockers([]);
        setContextOpen(false);
        setMenuOpen(false);
        if (untrack(() => props.live) === false) return;

        // Replay the persisted events from agents.events (the durable record).
        void replayRunEvents(runId)
          .then((replayed) => {
            if (stopped) return;
            const snapshot = eventsForRun(replayed, runId);
            setProvidedEvents((current) => mergeEvents(current, snapshot));
            setEvents(mergeEvents(providedEvents(), streamEvents()));
          })
          .catch(() => undefined);

        let stopped = false;
        const frames = coalesce<AgentEvent>((items) => {
          const nextStreamEvents = mergeEvents(streamEvents(), items);
          setStreamEvents(nextStreamEvents);
          setEvents(mergeEvents(providedEvents(), nextStreamEvents));
        });
        let detach = () => undefined;
        onCleanup(() => {
          stopped = true;
          frames.stop();
          detach();
        });
        void streamFromCore((event) => {
          if (!stopped && event.runId === runId) frames.push(event);
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
      },
      { defer: false },
    ),
  );

  createEffect(
    on(
      () => props.events,
      (incoming) => {
        if (!incoming) return;
        const snapshot = eventsForRun(incoming, props.runId);
        setProvidedEvents(snapshot);
        setEvents(mergeEvents(snapshot, streamEvents()));
      },
    ),
  );
  createEffect(
    on(
      () => props.findings,
      (incoming) =>
        setLocalFindings(mergeFindings(incoming ?? [], pinnedFindings())),
    ),
  );
  createEffect(
    on(
      () => props.session?.name,
      (name) => {
        if (!nameEdited()) setSessionName(name ?? DEFAULT_SESSION.name);
      },
    ),
  );

  const closeOverflowMenu = (returnFocus: boolean) => {
    setMenuOpen(false);
    if (returnFocus)
      pane
        ?.querySelector<HTMLButtonElement>(
          "[data-testid='agent-overflow-toggle']",
        )
        ?.focus();
  };
  const closeContextView = (returnFocus: boolean) => {
    setContextOpen(false);
    if (returnFocus)
      pane
        ?.querySelector<HTMLButtonElement>(
          "[data-testid='agent-context-toggle']",
        )
        ?.focus();
  };
  // The header popovers are plain <Show> blocks, so Escape and outside
  // pointer presses are wired here where their open state lives. Escape
  // returns focus to the control that opened the popover; outside presses
  // close without moving focus, and presses on the trigger itself are left
  // for its own toggle handler.
  createEffect(() => {
    if (!menuOpen() && !contextOpen()) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      if (menuOpen()) closeOverflowMenu(true);
      else if (contextOpen()) closeContextView(true);
    };
    const onPointerDown = (event: PointerEvent) => {
      if (!(event.target instanceof Node) || !pane) return;
      if (menuOpen()) {
        const menu = pane.querySelector(".agent-overflow-menu");
        const toggle = pane.querySelector(
          "[data-testid='agent-overflow-toggle']",
        );
        if (!menu?.contains(event.target) && !toggle?.contains(event.target))
          closeOverflowMenu(false);
      }
      if (contextOpen()) {
        const view = pane.querySelector(".agent-context-view");
        const chip = pane.querySelector("[data-testid='agent-context-toggle']");
        if (!view?.contains(event.target) && !chip?.contains(event.target))
          closeContextView(false);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("pointerdown", onPointerDown);
    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("pointerdown", onPointerDown);
    });
  });

  const session = (): AgentPaneSession => {
    const viewModel = props.viewModel;
    if (!viewModel) return props.session ?? DEFAULT_SESSION;
    const provided = props.session ?? DEFAULT_SESSION;
    return {
      ...provided,
      project: props.session?.project ?? viewModel.projectId,
      task: viewModel.taskId ?? provided.task,
      workflow: viewModel.workflowDefId ?? provided.workflow,
      context: viewModel.context,
      permissionPosture: viewModel.permissionPosture,
      status: viewModel.liveStatus,
    };
  };
  const panelSession = () => ({
    ...session(),
    name: sessionName() || session().name,
  });
  const posture = () =>
    props.viewModel?.permissionPosture ??
    props.permissionPosture ??
    session().permissionPosture;
  const researchOpen = () => props.researchOpen ?? internalResearchOpen();
  const costIsVisible = () => props.showCost ?? internalCostVisible();
  const contextPercent = () => {
    const context = session().context;
    return context && context.total > 0
      ? (context.used / context.total) * 100
      : 0;
  };
  const thinkingDisplay = () =>
    props.thinkingDisplay ?? internalThinkingDisplay();
  const toolCallsDisplay = () =>
    props.toolCallsDisplay ?? internalToolCallsDisplay();
  const gateMode = () => props.gateMode ?? internalGateMode();
  const updateGateMode = (mode: AgentGateMode) => {
    props.onGateModeChange?.(mode);
    if (props.gateMode === undefined) setInternalGateMode(mode);
  };
  const updateThinkingDisplay = (display: AgentThinkingDisplay) => {
    props.onThinkingDisplayChange?.(display);
    if (props.thinkingDisplay === undefined)
      setInternalThinkingDisplay(display);
  };
  const updateToolCallsDisplay = (display: AgentToolDisplay) => {
    props.onToolCallsDisplayChange?.(display);
    if (props.toolCallsDisplay === undefined)
      setInternalToolCallsDisplay(display);
  };
  const toggleCost = () => {
    const visible = !costIsVisible();
    props.onCostVisibilityChange?.(visible);
    if (props.showCost === undefined) setInternalCostVisible(visible);
  };
  const toggleResearch = () => {
    const next = !researchOpen();
    props.onResearchToggle?.(next);
    if (!props.onResearchToggle) setInternalResearchOpen(next);
  };
  const activePlan = createMemo<AgentPanePlan | undefined>(() => {
    const updates = props.planUpdates;
    return updates?.length
      ? updates[updates.length - 1]
      : (props.plan ?? props.viewModel?.activePlan);
  });
  const blockers = createMemo<AgentPaneBlocker[]>(() => {
    const resolved = new Set(resolvedBlockers());
    const candidates = [
      ...(props.blockers ?? []),
      ...events()
        .filter(
          (event) =>
            posture() === "gated" && event.verb === "permission_request",
        )
        .map((event) => ({
          id: event.id,
          kind: "gate" as const,
          title: "Permission request",
          detail: eventText(event),
          event,
        })),
    ];
    const unique = new Map<string, AgentPaneBlocker>();
    for (const blocker of candidates) {
      if (!resolved.has(blocker.id)) unique.set(blocker.id, blocker);
    }
    return [...unique.values()];
  });
  const pendingBlockerCount = createMemo(
    () => blockers().length + (props.elicitation ? 1 : 0),
  );
  const blockerExpanded = createMemo(
    () =>
      blockers().some((blocker) => !minimizedBlockers().includes(blocker.id)) ||
      Boolean(
        props.elicitation &&
          !minimizedBlockers().includes(props.elicitation.id),
      ),
  );
  const status = createMemo<AgentPanelStatus>(() => {
    if (pendingBlockerCount()) return "waiting";
    return (
      session().status ??
      (events().some((event) => event.verb === "tool_call")
        ? "working"
        : "idle")
    );
  });
  const running = () => status() === "working";
  const scrollToBlocker = () => {
    pane
      ?.querySelector<HTMLElement>(
        "[data-blocker-id], [data-testid='agent-elicitation']",
      )
      ?.scrollIntoView?.({ block: "center" });
  };
  const pinCitation = (citation: AgentPaneCitation) => {
    const finding: AgentPaneFinding = {
      id: citation.id,
      title: citation.label,
      summary: citation.summary ?? citation.source,
      source: citation.source,
      provenance: "this_run",
    };
    setPinnedFindings((current) =>
      current.some((item) => item.id === finding.id)
        ? current
        : [...current, finding],
    );
    setLocalFindings((current) => mergeFindings(current, [finding]));
    props.onPinCitation?.(citation);
  };
  const reviewFinding = (finding: AgentPaneFinding) => {
    setReviewedFindings((current) =>
      current.includes(finding.id) ? current : [...current, finding.id],
    );
    props.onReviewFinding?.(finding);
  };
  const resolvePermission = (
    event: AgentEvent,
    action: "approve" | "decline" | "remaining",
  ) => {
    setResolvedBlockers((current) =>
      current.includes(event.id) ? current : [...current, event.id],
    );
    if (action === "approve") props.onApprovePermission?.(event);
    if (action === "decline") props.onDeclinePermission?.(event);
    if (action === "remaining") props.onApproveRemainingTurn?.(event);
  };

  return (
    <section
      ref={pane}
      class="agent-pane"
      data-testid="agent-pane"
      data-run-id={props.runId}
      data-pty="false"
      data-research-open={researchOpen() ? "true" : "false"}
      data-permission-posture={posture()}
      data-blocker-expanded={blockerExpanded() ? "true" : "false"}
      data-context-warning={contextPercent() > 80 ? "true" : "false"}
    >
      <AgentHeader
        session={panelSession()}
        posture={posture()}
        costVisible={costIsVisible()}
        contextOpen={contextOpen()}
        researchOpen={researchOpen()}
        showResearchControl={props.showResearchControl !== false}
        menuOpen={menuOpen()}
        gateMode={gateMode()}
        toolCallsDisplay={toolCallsDisplay()}
        onCostToggle={toggleCost}
        onGateModeChange={updateGateMode}
        onToolCallsDisplay={updateToolCallsDisplay}
        onContextToggle={() => setContextOpen((value) => !value)}
        onResearchToggle={toggleResearch}
        onMenuToggle={() => setMenuOpen((value) => !value)}
        onNewSession={props.onNewSession}
        onCompact={props.onCompact}
        onClearContext={props.onClearContext}
        onSessionRename={(name) => {
          setNameEdited(true);
          setSessionName(name);
          props.onSessionRename?.(name);
        }}
        onHarnessChange={props.onHarnessChange}
        onModelChange={props.onModelChange}
        onEffortChange={props.onEffortChange}
      />
      <Show when={contextOpen()}>
        <section class="agent-context-view" data-testid="agent-context-view">
          <strong>Context window</strong>
          <span>Memory catalog · returned tool docs · session research</span>
          <Show
            when={session().context}
            fallback={<code>Context usage unavailable</code>}
          >
            {(context) => (
              <code>
                {formatTokens(context().used)} used of{" "}
                {formatTokens(context().total)}
              </code>
            )}
          </Show>
        </section>
      </Show>
      <div class="agent-pane-layout">
        <main class="agent-pane-main">
          <button
            type="button"
            class={`agent-live-pill agent-live-${status()}`}
            data-testid="agent-live-status"
            onClick={scrollToBlocker}
          >
            <span class="agent-live-dot" /> {status()}
            <Show when={pendingBlockerCount()}>
              <small> · {pendingBlockerCount()} needs you</small>
            </Show>
          </button>
          <Show when={streamError()}>
            {(error) => (
              <InlineError
                cause={error()}
                next="Check the core ACP event stream and reopen this pane."
              />
            )}
          </Show>
          <Show when={pendingBlockerCount()}>
            <BlockerStack
              blockers={blockers()}
              elicitation={props.elicitation}
              minimized={minimizedBlockers()}
              onToggle={(id) =>
                setMinimizedBlockers((current) =>
                  current.includes(id)
                    ? current.filter((item) => item !== id)
                    : [...current, id],
                )
              }
              onAcceptElicitation={props.onAcceptElicitation}
              onDeclineElicitation={props.onDeclineElicitation}
              onCancelElicitation={props.onCancelElicitation}
            />
          </Show>
          <div
            class="agent-stream-shell"
            data-scrim={blockerExpanded() ? "true" : "false"}
          >
            <section
              class="agent-stream"
              data-testid="agent-stream"
              aria-live="polite"
            >
              <AgentEventStream
                events={events()}
                posture={posture()}
                thinkingDisplay={thinkingDisplay()}
                toolCallsDisplay={toolCallsDisplay()}
                resolved={resolvedBlockers()}
                onThinkingDisplay={updateThinkingDisplay}
                onApprove={(event) => resolvePermission(event, "approve")}
                onDecline={(event) => resolvePermission(event, "decline")}
                onApproveRemaining={(event) =>
                  resolvePermission(event, "remaining")
                }
                onResubmit={props.onResubmit}
                onCopyPrompt={props.onCopyPrompt}
                onOpenFile={props.onOpenFile}
                onPinCitation={pinCitation}
              />
              <Show when={props.checkpoints?.length}>
                <CheckpointMarkers
                  checkpoints={props.checkpoints ?? []}
                  restored={restoredCheckpoint()}
                  onRestore={(checkpoint) => {
                    setRestoredCheckpoint(checkpoint.id);
                    props.onRestoreCheckpoint?.(checkpoint);
                  }}
                  onUndo={(checkpoint) => {
                    setRestoredCheckpoint(null);
                    props.onUndoCheckpoint?.(checkpoint);
                  }}
                  onOpenFile={props.onOpenFile}
                />
              </Show>
            </section>
          </div>
          <Show when={activePlan()}>
            {(plan) => (
              <PlanDock
                plan={plan()}
                forceCollapsed={blockerExpanded()}
                onOpenFile={props.onOpenFile}
              />
            )}
          </Show>
          <Composer
            running={running()}
            value={composerValue()}
            onValue={setComposerValue}
            onSend={props.onSend}
            onQueue={props.onQueue}
            onStop={props.onStop}
            mentionSuggestions={props.mentionSuggestions}
            onSlashCommand={(command) => {
              if (command === "new-session") props.onNewSession?.();
              if (command === "compact") props.onCompact?.();
              if (command === "clear-context") props.onClearContext?.();
              if (command === "context") {
                setContextOpen(true);
                props.onViewContext?.();
              }
            }}
          />
        </main>
        <Show when={researchOpen() && props.showResearchPane !== false}>
          <ResearchPane
            sessionId={props.viewModel?.sessionId ?? props.runId}
            findings={localFindings()}
            reviewed={reviewedFindings()}
            onReview={reviewFinding}
            onPromote={props.onPromoteFinding}
            onOpenFile={props.onOpenFile}
          />
        </Show>
      </div>
    </section>
  );
}
