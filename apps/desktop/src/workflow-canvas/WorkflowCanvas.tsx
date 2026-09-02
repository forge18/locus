import "./polyfills";
import "@dschz/solid-flow/styles";
import {
    Background,
    Controls,
    Handle,
    MarkerType,
    Position,
    SolidFlow,
    createEdgeStore,
    createNodeStore,
    useViewport,
} from "@dschz/solid-flow";
import type { NodeProps, NodeTypes } from "@dschz/solid-flow";
import { createEffect, createSignal, For, Show } from "solid-js";
import type { CanvasEdge, CanvasNode } from "../data/workflow";
import { NO_MODEL_NOTE } from "../data/workflow";
import type { NormalizedWorkflowEvent } from "../data/workflow-events";

const NODE_W = 168;
const NODE_H = 46;

interface FlowNodeData extends Record<string, unknown> {
    kind: string;
    label: string;
    state: string;
    tone: "default" | "condition";
    inputs: string[];
    outputs: string[];
}

export interface WorkflowCanvasProps {
    nodes: CanvasNode[];
    edges: CanvasEdge[];
    loop: {
        x: number;
        y: number;
        width: number;
        height: number;
        label: string;
    };
    events: NormalizedWorkflowEvent[];
    onSelect?: (id: string) => void;
    onDrop?: (kind: string, position: { x: number; y: number }) => void;
    onConnect?: (edge: CanvasEdge) => void;
    onNodePositionChange?: (
        id: string,
        position: { x: number; y: number },
    ) => void;
}

export const NODE_HANDLES = {
    goal: { inputs: ["approved"], outputs: ["start"] },
    agent: { inputs: ["in"], outputs: ["out"] },
    task: { inputs: ["in"], outputs: ["out"] },
    loop: { inputs: ["in"], outputs: ["body", "exit"] },
    condition: { inputs: ["in"], outputs: ["true", "false"] },
    gate: { inputs: ["in"], outputs: ["pass", "reject"] },
    verify: { inputs: ["in"], outputs: ["passed", "failed"] },
} as const;

export const inputHandles = (kind = "agent"): string[] => [
    ...(NODE_HANDLES[kind as keyof typeof NODE_HANDLES]?.inputs ?? ["in"]),
];

export function outputHandles(kind: string): string[] {
    switch (kind.toLowerCase()) {
        case "goal":
            return [...NODE_HANDLES.goal.outputs];
        case "condition":
            return [...NODE_HANDLES.condition.outputs];
        case "gate":
            return [...NODE_HANDLES.gate.outputs];
        case "verify":
            return [...NODE_HANDLES.verify.outputs];
        case "loop":
            return [...NODE_HANDLES.loop.outputs];
        default:
            return [...NODE_HANDLES.agent.outputs];
    }
}

function handleOffset(index: number, count: number): string {
    return `${((index + 1) / (count + 1)) * 100}%`;
}

function FlowNodeView(props: NodeProps<FlowNodeData>) {
    return (
        <div
            class={`wf-flow-node ${props.data.tone === "condition" ? "wf-node-condition" : ""}`}
            data-testid={`wf-node-${props.id}`}
            data-tone={props.data.tone}
            data-node-kind={props.data.kind}
            data-selected={props.selected ? "true" : "false"}
        >
            <For each={props.data.inputs}>
                {(handle, index) => (
                    <Handle
                        type="target"
                        position={Position.Left}
                        id={handle}
                        data-handle-id={handle}
                        style={{
                            top: handleOffset(
                                index(),
                                props.data.inputs.length,
                            ),
                        }}
                    />
                )}
            </For>
            <div
                class="wf-node-strip"
                data-testid={`wf-node-strip-${props.id}`}
            >
                <span class="wf-node-kind">{props.data.kind}</span>
                <span
                    class="wf-node-state"
                    data-testid={`wf-node-state-${props.id}`}
                >
                    {props.data.state}
                </span>
            </div>
            <div class="wf-node-label">{props.data.label}</div>
            <For each={props.data.outputs}>
                {(handle, index) => (
                    <Handle
                        type="source"
                        position={Position.Right}
                        id={handle}
                        data-handle-id={handle}
                        style={{
                            top: handleOffset(
                                index(),
                                props.data.outputs.length,
                            ),
                        }}
                    />
                )}
            </For>
        </div>
    );
}

export const GoalNode = FlowNodeView;
export const AgentNode = FlowNodeView;
export const TaskNode = FlowNodeView;
export const LoopNode = FlowNodeView;
export const ConditionNode = FlowNodeView;
export const GateNode = FlowNodeView;
export const VerifyNode = FlowNodeView;

export const nodeTypes = {
    agent: AgentNode,
    task: TaskNode,
    loop: LoopNode,
    condition: ConditionNode,
    gate: GateNode,
    verify: VerifyNode,
} as unknown as NodeTypes;

function sourceHandle(edge: CanvasEdge): string {
    if (edge.from === "n-cond") return edge.dashed ? "false" : "true";
    if (edge.from === "n-gate") return "pass";
    if (edge.from === "n-verify") return "passed";
    return "out";
}

function markerFor(edge: CanvasEdge): string {
    if (edge.dashed) return "arrow-loop";
    if (edge.label?.startsWith("passed") || edge.label === "approved") {
        return "arrow-pass";
    }
    if (edge.label?.startsWith("failed")) return "arrow-fail";
    return "arrow-default";
}

function flowNodes(nodes: CanvasNode[], events: NormalizedWorkflowEvent[]) {
    const stateByNode = new Map(
        events.map((event) => [event.nodeId, event.state]),
    );
    return nodes.map((node) => ({
        id: node.id,
        type: node.kind.toLowerCase(),
        position: { x: node.x, y: node.y },
        data: {
            kind: node.kind,
            label: node.label,
            state: stateByNode.get(node.id) ?? node.state,
            tone: node.tone,
            inputs: inputHandles(node.kind),
            outputs: outputHandles(node.kind),
        },
        domAttributes: { "data-testid": `wf-flow-node-${node.id}` },
    }));
}

function markerColor(marker: string): string {
    switch (marker) {
        case "arrow-pass":
            return "status-success";
        case "arrow-fail":
            return "action-attention";
        default:
            return "border-strong";
    }
}

function flowEdges(edges: CanvasEdge[]) {
    return edges.map((edge) => {
        const marker = markerFor(edge);
        return {
            id: `${edge.from}-${edge.to}`,
            source: edge.from,
            sourceHandle: sourceHandle(edge),
            target: edge.to,
            targetHandle: "in",
            type: "default",
            label: edge.label ?? undefined,
            class: edge.dashed ? "wf-flow-edge-loop" : undefined,
            style: {
                stroke: `var(--${markerColor(marker)})`,
            },
            markerEnd: {
                type: MarkerType.ArrowClosed,
                color: `var(--${markerColor(marker)})`,
            },
        };
    });
}

function LoopGroup(props: { loop: WorkflowCanvasProps["loop"] }) {
    const viewport = useViewport();
    return (
        <div
            class="wf-loop-group"
            data-testid="wf-loop-group"
            style={{
                left: `${props.loop.x}px`,
                top: `${props.loop.y}px`,
                width: `${props.loop.width}px`,
                height: `${props.loop.height}px`,
                transform: `translate(${viewport().x}px, ${viewport().y}px) scale(${viewport().zoom})`,
            }}
        >
            <span class="wf-loop-label">{props.loop.label}</span>
        </div>
    );
}

export function WorkflowCanvas(props: WorkflowCanvasProps) {
    const [flowNodesStore, setFlowNodesStore] = createNodeStore(
        flowNodes(props.nodes, props.events) as never[],
    );
    const [flowEdgesStore, setFlowEdgesStore] = createEdgeStore(
        flowEdges(props.edges) as never[],
    );
    const [zoom, setZoom] = createSignal(1);

    // Solid Flow owns the interactive stores. Keep them synchronized with the
    // authored graph so a drop, selection, event update, or parent refresh is
    // reflected without remounting the canvas.
    createEffect(() => {
        setFlowNodesStore(flowNodes(props.nodes, props.events) as never[]);
        setFlowEdgesStore(flowEdges(props.edges) as never[]);
    });

    const byId = () => new Map(props.nodes.map((node) => [node.id, node]));
    const centre = (id: string) => {
        const node = byId().get(id);
        return {
            x: (node?.x ?? 0) + NODE_W / 2,
            y: (node?.y ?? 0) + NODE_H / 2,
        };
    };
    const onDrop = (event: DragEvent) => {
        event.preventDefault();
        const kind =
            event.dataTransfer?.getData("application/x-locus-node") ||
            event.dataTransfer?.getData("text/plain");
        if (!kind || !props.onDrop) return;
        const target = event.currentTarget as HTMLElement;
        const bounds = target.getBoundingClientRect();
        props.onDrop(kind, {
            x: Math.max(0, event.clientX - bounds.left),
            y: Math.max(0, event.clientY - bounds.top),
        });
    };
    const onConnect = (connection: { source: string; target: string }) => {
        props.onConnect?.({
            from: connection.source,
            to: connection.target,
            label: null,
            dashed: false,
        });
    };

    return (
        <div
            class="wf-canvas"
            data-testid="wf-canvas"
            onDragOver={(event) => event.preventDefault()}
            onDrop={onDrop}
        >
            <div class="wf-flow-engine" data-testid="wf-solid-flow">
                <SolidFlow
                    nodes={flowNodesStore as never}
                    edges={flowEdgesStore as never}
                    nodeTypes={nodeTypes}
                    fitView={false}
                    panOnScroll
                    selectionKey="Shift"
                    onNodeClick={(params) => {
                        // solid-flow's click declaration uses targetNode while
                        // its runtime callback supplies node.
                        const event = params as unknown as {
                            node?: { id: string };
                            targetNode?: { id: string } | null;
                        };
                        const node = event.node ?? event.targetNode;
                        if (node) props.onSelect?.(node.id);
                    }}
                    onNodeDragStop={({ targetNode }) => {
                        if (targetNode)
                            props.onNodePositionChange?.(
                                targetNode.id,
                                targetNode.position,
                            );
                    }}
                    onConnect={onConnect}
                    onMove={(_, viewport) => setZoom(viewport.zoom)}
                >
                    <Background />
                    <Controls />
                    <LoopGroup loop={props.loop} />
                </SolidFlow>
            </div>
            <svg class="wf-edges" data-testid="wf-edges" aria-hidden="true">
                <defs>
                    <For
                        each={[
                            "arrow-default",
                            "arrow-pass",
                            "arrow-fail",
                            "arrow-loop",
                        ]}
                    >
                        {(marker) => (
                            <marker
                                id={marker}
                                data-testid={`wf-marker-${marker}`}
                                viewBox="0 0 8 8"
                                refX="7"
                                refY="4"
                                markerWidth="6"
                                markerHeight="6"
                                orient="auto"
                            >
                                <path
                                    d="M0,0 L8,4 L0,8 z"
                                    fill="currentColor"
                                />
                            </marker>
                        )}
                    </For>
                </defs>
                <For each={props.edges}>
                    {(edge) => {
                        const a = centre(edge.from);
                        const b = centre(edge.to);
                        const marker = markerFor(edge);
                        return (
                            <line
                                class="graph-edge"
                                data-testid={`wf-edge-${edge.from}-${edge.to}`}
                                data-dashed={edge.dashed ? "true" : undefined}
                                x1={a.x}
                                y1={a.y}
                                x2={b.x}
                                y2={b.y}
                                stroke-dasharray={
                                    edge.dashed ? "4 3" : undefined
                                }
                                marker-end={`url(#${marker})`}
                            />
                        );
                    }}
                </For>
            </svg>
            <For each={props.edges.filter((edge) => edge.label)}>
                {(edge) => {
                    const a = centre(edge.from);
                    const b = centre(edge.to);
                    return (
                        <span
                            class="wf-edge-label"
                            data-testid={`wf-edge-label-${edge.from}-${edge.to}`}
                            style={{
                                left: `${(a.x + b.x) / 2 - 20}px`,
                                top: `${(a.y + b.y) / 2 - 8}px`,
                            }}
                        >
                            {edge.label}
                        </span>
                    );
                }}
            </For>
            <div class="wf-canvas-foot">
                <span class="wf-zoom" data-testid="wf-zoom">
                    {`${Math.round(zoom() * 100)}%`}
                </span>
                <span class="wf-canvas-note" data-testid="wf-no-model-note">
                    {NO_MODEL_NOTE}
                </span>
                <Show when={props.events.length > 0}>
                    <span
                        class="wf-live-overlay"
                        data-testid="wf-live-overlay"
                        data-event-source="workflow-events"
                    >
                        live · {props.events.length} normalized events
                    </span>
                </Show>
            </div>
        </div>
    );
}
