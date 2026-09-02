import { LOOP_GROUP } from "./demo/fixtures/workflow";
import type {
        CanvasEdge,
        CanvasNode,
        Clause,
        Guardrail,
        PaletteNode,
        Preset,
} from "./demo/fixtures/workflow";
import { workflowEventsForTranscript } from "./workflow-events";
import type { NormalizedWorkflowEvent } from "./workflow-events";
import type { GuardrailTrip, WorkflowDef } from "../types/workflows";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

export {
        ARROW_MARKERS,
        BUDGET_NOTE,
        COMPILED,
        COMPILED_NOTE,
        LOOP_GROUP,
        NO_MODEL_NOTE,
        OPERAND_NOTE,
        PRESET_NOTE,
        WAITING_NOTE,
        ZOOM,
} from "./demo/fixtures/workflow";
export type {
        CanvasEdge,
        CanvasNode,
        Clause,
        Guardrail,
        PaletteNode,
        Preset,
} from "./demo/fixtures/workflow";
export type { NormalizedWorkflowEvent } from "./workflow-events";

export interface WorkflowDefinitionSummary {
        id: string;
        name: string;
        version: number;
}

export interface PersistedWorkflowDefinition extends WorkflowDefinitionSummary {
        projectId: string;
        graph: Record<string, unknown>;
        spec: Record<string, unknown>;
        verifyCommand: string;
}

export interface WorkflowDefinitionSaveRequest {
        projectId: string;
        name: string;
        graph: Record<string, unknown>;
        governance: Record<string, unknown>;
}

export function readGuardrailBody(value: Record<string, unknown>): string {
        return typeof value["prompt"] === "string" ? value["prompt"] : "";
}

export function writeGuardrail(
        name: string,
        value: string,
): Record<string, string> {
        return { name, ["prompt"]: value };
}

export function fetchWorkflowPalette(): Promise<Envelope<PaletteNode[]>> {
        return dataProvider().query<PaletteNode>("workflow_node_vocabulary");
}

export function fetchWorkflowPresets(): Promise<Envelope<Preset[]>> {
        return dataProvider().query<Preset>("workflow_presets");
}

export function fetchWorkflowOperands(): Promise<Envelope<string[]>> {
        return dataProvider().query<string>("condition_operands");
}

export function fetchWorkflowDefinitions(
        projectId: string,
): Promise<Envelope<WorkflowDefinitionSummary[]>> {
        return dataProvider().query<WorkflowDefinitionSummary>(
                "workflow_definitions",
                {
                        projectId,
                },
        );
}

export function fetchWorkflowDefinition(
        projectId: string,
        workflowId: string,
): Promise<Envelope<PersistedWorkflowDefinition>> {
        return dataProvider().queryOne<PersistedWorkflowDefinition>(
                "workflow_definition_detail",
                { projectId, workflowId },
        );
}

export function saveWorkflowDefinition(
        request: WorkflowDefinitionSaveRequest,
): Promise<Envelope<PersistedWorkflowDefinition>> {
        return dataProvider().queryOne<PersistedWorkflowDefinition>(
                "workflow_definition_save",
                { request },
        );
}

/** Becomes: invoke("workflow_node_vocabulary") */
export function usePalette(): PaletteNode[] {
        return (
                dataProvider().read?.<PaletteNode[]>(
                        "workflow_node_vocabulary",
                ) ?? []
        );
}

/** Becomes: invoke("workflow_presets") */
export function usePresets(): Preset[] {
        return dataProvider().read?.<Preset[]>("workflow_presets") ?? [];
}

/** Becomes: invoke("workflow_def", { id }) */
export function useWorkflow(): WorkflowDef {
        return (
                dataProvider().read?.<WorkflowDef>("workflow_def") ?? {
                        id: "",
                        name: "Unavailable workflow",
                        version: 0,
                        nodes: [],
                        edges: [],
                        spec: "",
                        updatedAt: "",
                }
        );
}

/** Becomes: invoke("workflow_graph", { id }) */
export function useCanvas(): {
        nodes: CanvasNode[];
        edges: CanvasEdge[];
        loop: typeof LOOP_GROUP;
        markers: readonly string[];
        events: NormalizedWorkflowEvent[];
} {
        return (
                dataProvider().read?.<{
                        nodes: CanvasNode[];
                        edges: CanvasEdge[];
                        loop: typeof LOOP_GROUP;
                        markers: readonly string[];
                        events: NormalizedWorkflowEvent[];
                }>("workflow_graph") ?? {
                        nodes: [],
                        edges: [],
                        loop: LOOP_GROUP,
                        markers: [],
                        events: [],
                }
        );
}

/** Becomes: invoke("condition_expression", { nodeId }) */
export function useExpression(): Clause[] {
        return dataProvider().read?.<Clause[]>("condition_expression") ?? [];
}

/** Becomes: invoke("condition_operands") — every one is a column. */
export function useOperands(): readonly string[] {
        return dataProvider().read?.<string[]>("condition_operands") ?? [];
}

/** Becomes: invoke("workflow_guardrails", { id }) */
export function useGuardrails(): Guardrail[] {
        return dataProvider().read?.<Guardrail[]>("workflow_guardrails") ?? [];
}

/** Becomes: subscribe("workflow.events", { id }) */
export function useWorkflowEvents(): NormalizedWorkflowEvent[] {
        return workflowEventsForTranscript();
}

/** Becomes: emit("guardrail_tripped") */
export function useGuardrailTrips(): GuardrailTrip[] {
        return dataProvider().read?.<GuardrailTrip[]>("guardrail_trips") ?? [];
}
