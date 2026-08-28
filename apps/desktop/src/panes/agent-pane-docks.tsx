import { For, Show, createSignal } from "solid-js";
import type {
    AgentPaneBlocker,
    AgentPaneCheckpoint,
    AgentPaneElicitation,
    AgentPaneFinding,
    AgentPanePlan,
    AgentPaneProps,
} from "./agent-panel-model";
import { FileLink, RichContent } from "./agent-pane-content";
import { ElicitationCard } from "./agent-pane-controls";

export function DockedBlocker(props: {
    blocker: AgentPaneBlocker;
    minimized: boolean;
    onToggle: () => void;
}) {
    return (
        <article
            class="agent-blocker"
            data-blocker-kind={props.blocker.kind}
            data-blocker-id={props.blocker.id}
            data-blocker-minimized={props.minimized}
        >
            <header>
                <span class="agent-card-kind">{props.blocker.kind}</span>
                <button
                    type="button"
                    aria-expanded={!props.minimized}
                    onClick={props.onToggle}
                >
                    {props.minimized ? "Restore" : "Minimize"}
                </button>
            </header>
            <Show when={!props.minimized}>
                <strong>{props.blocker.title}</strong>
                <p>{props.blocker.detail}</p>
                <Show when={props.blocker.event}>
                    <p class="agent-blocker-hint">
                        Review the inline diff in the stream before responding.
                    </p>
                </Show>
            </Show>
        </article>
    );
}

export function BlockerStack(props: {
    blockers: AgentPaneBlocker[];
    elicitation?: AgentPaneElicitation;
    minimized: string[];
    onToggle: (id: string) => void;
    onAcceptElicitation?: AgentPaneProps["onAcceptElicitation"];
    onDeclineElicitation?: AgentPaneProps["onDeclineElicitation"];
    onCancelElicitation?: AgentPaneProps["onCancelElicitation"];
}) {
    return (
        <section class="agent-docked-blockers" data-testid="agent-blocker">
            <For each={props.blockers}>
                {(blocker) => (
                    <DockedBlocker
                        blocker={blocker}
                        minimized={props.minimized.includes(blocker.id)}
                        onToggle={() => props.onToggle(blocker.id)}
                    />
                )}
            </For>
            <Show when={props.elicitation}>
                {(elicitation) => (
                    <ElicitationCard
                        elicitation={elicitation()}
                        minimized={props.minimized.includes(elicitation().id)}
                        onToggle={() => props.onToggle(elicitation().id)}
                        onAccept={props.onAcceptElicitation}
                        onDecline={props.onDeclineElicitation}
                        onCancel={props.onCancelElicitation}
                    />
                )}
            </Show>
        </section>
    );
}

export function PlanDock(props: {
    plan: AgentPanePlan;
    forceCollapsed?: boolean;
    onOpenFile?: (path: string) => void;
}) {
    const [open, setOpen] = createSignal(false);
    const complete = () =>
        props.plan.steps.filter(
            (step) => step.status === "done" || step.status === "completed",
        ).length;
    return (
        <section
            class="agent-plan-dock"
            data-testid="agent-plan-dock"
            data-plan-id={props.plan.id}
            data-forced-collapsed={props.forceCollapsed ? "true" : "false"}
        >
            <header>
                <button
                    type="button"
                    aria-expanded={open() && !props.forceCollapsed}
                    onClick={() => setOpen((value) => !value)}
                >
                    <span class="agent-card-kind">plan</span>
                    <strong>{props.plan.title}</strong>
                </button>
                <span>
                    {complete()}/{props.plan.steps.length}
                </span>
            </header>
            <Show when={open() && !props.forceCollapsed}>
                <ol>
                    <For each={props.plan.steps}>
                        {(step) => (
                            <li data-step-status={step.status}>
                                <span class="agent-step-mark" />
                                <span>{step.title}</span>
                                <small>{step.outcome ?? step.status}</small>
                            </li>
                        )}
                    </For>
                </ol>
                <Show when={props.plan.markdown}>
                    {(markdown) => (
                        <RichContent
                            text={markdown()}
                            onOpenFile={props.onOpenFile}
                        />
                    )}
                </Show>
                <Show when={props.plan.file}>
                    {(file) => (
                        <FileLink path={file()} onOpenFile={props.onOpenFile} />
                    )}
                </Show>
                <Show when={props.plan.outcome}>
                    <p class="agent-plan-outcome">{props.plan.outcome}</p>
                </Show>
            </Show>
        </section>
    );
}

export function CheckpointMarkers(props: {
    checkpoints: AgentPaneCheckpoint[];
    restored: string | null;
    onRestore?: (checkpoint: AgentPaneCheckpoint) => void;
    onUndo?: (checkpoint: AgentPaneCheckpoint) => void;
    onOpenFile?: (path: string) => void;
}) {
    return (
        <section class="agent-checkpoints" data-testid="agent-checkpoints">
            <For each={props.checkpoints}>
                {(checkpoint) => {
                    const restored = () =>
                        props.restored === checkpoint.id ||
                        checkpoint.state === "restored";
                    return (
                        <div
                            class="agent-checkpoint"
                            data-checkpoint-id={checkpoint.id}
                            data-checkpoint-state={
                                restored() ? "restored" : checkpoint.state
                            }
                        >
                            <span class="agent-checkpoint-marker" />
                            <div>
                                <strong>{checkpoint.label}</strong>
                                <FileLink
                                    path={checkpoint.file}
                                    onOpenFile={props.onOpenFile}
                                />
                            </div>
                            <Show
                                when={restored()}
                                fallback={
                                    <button
                                        type="button"
                                        onClick={() =>
                                            props.onRestore?.(checkpoint)
                                        }
                                    >
                                        Restore
                                    </button>
                                }
                            >
                                <button
                                    type="button"
                                    onClick={() => props.onUndo?.(checkpoint)}
                                >
                                    Undo
                                </button>
                            </Show>
                        </div>
                    );
                }}
            </For>
            <Show when={props.restored}>
                <p class="agent-restored-banner" role="status">
                    Workspace restored. The transcript remains intact.
                </p>
            </Show>
        </section>
    );
}

export function ResearchPane(props: {
    sessionId: string;
    findings: AgentPaneFinding[];
    reviewed: string[];
    onReview: (finding: AgentPaneFinding) => void;
    onPromote?: (finding: AgentPaneFinding) => void;
    onOpenFile?: (path: string) => void;
}) {
    const isReviewed = (finding: AgentPaneFinding) =>
        finding.reviewed || props.reviewed.includes(finding.id);
    return (
        <aside
            class="agent-research-pane"
            data-testid="agent-research-pane"
            data-session-id={props.sessionId}
        >
            <header>
                <div>
                    <span class="agent-pane-eyebrow">session research</span>
                    <h2>Findings</h2>
                </div>
                <span>{props.findings.length} items</span>
            </header>
            <p class="agent-research-note">
                Sources and summaries for this session. Seeds are inherited,
                never promoted automatically.
            </p>
            <p class="agent-research-floor">
                Research tools: fixed session floor · independent of permission
                mode.
            </p>
            <Show
                when={props.findings.length}
                fallback={
                    <p class="agent-empty-state">
                        No findings yet. Research will appear here without
                        entering the project wiki.
                    </p>
                }
            >
                <div class="agent-findings">
                    <For each={props.findings}>
                        {(finding) => (
                            <article
                                class="agent-finding"
                                data-finding-id={finding.id}
                                data-provenance={finding.provenance}
                            >
                                <header>
                                    <span class="agent-provenance">
                                        {finding.provenance.replace(/_/g, " ")}
                                    </span>
                                    <code>{finding.id}</code>
                                </header>
                                <h3>{finding.title}</h3>
                                <p>{finding.summary}</p>
                                <FileLink
                                    path={finding.source}
                                    onOpenFile={props.onOpenFile}
                                />
                                <div class="agent-card-actions">
                                    <Show
                                        when={isReviewed(finding)}
                                        fallback={
                                            <button
                                                type="button"
                                                onClick={() =>
                                                    props.onReview(finding)
                                                }
                                            >
                                                Review for close
                                            </button>
                                        }
                                    >
                                        <span class="agent-reviewed">
                                            reviewed for close
                                        </span>
                                        <button
                                            type="button"
                                            onClick={() =>
                                                props.onPromote?.(finding)
                                            }
                                        >
                                            Promote at close
                                        </button>
                                    </Show>
                                </div>
                            </article>
                        )}
                    </For>
                </div>
            </Show>
        </aside>
    );
}
